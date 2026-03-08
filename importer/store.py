"""
Storage layer: converts LLM-extracted Pydantic objects into database rows.

All helpers accept a SQLAlchemy ``Session`` and flush (but do **not** commit)
after each insert so that auto-generated PKs are available immediately.
The caller is responsible for the final ``session.commit()``.

Text encoding
─────────────
The ``texts`` table stores content as raw UTF-8 bytes plus a binary
``formatting`` blob.  Each 8-byte formatting segment encodes:

  bytes 0-1 : language  (u16 LE)  – 0 = default
  bytes 2-5 : start     (u32 LE)  – byte offset into ``content``
  bytes 6-7 : flags     (u16 LE)  – bitfield (see FLAG_* constants)
"""

from __future__ import annotations

import struct

from sqlalchemy.orm import Session

from db_models import (
    Element,
    ElementKindEnum,
    ElementalProblem,
    ElementalProblemKindEnum,
    ElementalQuestion,
    ListItem,
    ListRow,
    OrderFormatEnum,
    OrderTypeEnum,
    Paragraph,
    ParagraphElement,
    ProblemCategory,
    QuestionBlockKindEnum,
    QuestionSeries,
    QuestionSeriesItem,
    SingleProblem,
    TextRow,
)
from schema import Content, ExtractedPage, ExtractedProblem, ExtractedQuestion

# ── Text flags (u16 LE, bits 6-7 of each formatting segment) ──────────────────
FLAG_BOLD = 1 << 0
FLAG_ITALIC = 1 << 1
FLAG_FORMULA = 1 << 8  # Non-standard extension: marks a run as inline LaTeX


# ══════════════════════════════════════════════════════════════════════════════
# Encoding helpers
# ══════════════════════════════════════════════════════════════════════════════


def _encode_content(content: Content) -> tuple[bytes, bytes]:
    """
    Encode a :class:`~schema.Content` block into the ``(formatting, content)``
    byte representation expected by the ``texts`` table.

    Returns:
        fmt_bytes  – concatenated 8-byte formatting segments (one per run)
        cnt_bytes  – raw UTF-8 payload of all runs concatenated
    """
    fmt_buf = bytearray()
    cnt_buf = bytearray()

    for run in content.runs:
        start = len(cnt_buf)
        raw = run.text.encode("utf-8")
        cnt_buf.extend(raw)

        lang: int = 0
        flags: int = 0
        if run.is_formula:
            flags |= FLAG_FORMULA
        if run.bold:
            flags |= FLAG_BOLD
        if run.italic:
            flags |= FLAG_ITALIC

        # 8-byte segment: u16 lang + u32 start + u16 flags (all little-endian)
        fmt_buf.extend(struct.pack("<HIH", lang, start, flags))

    return bytes(fmt_buf), bytes(cnt_buf)


# ══════════════════════════════════════════════════════════════════════════════
# Insert helpers
# ══════════════════════════════════════════════════════════════════════════════


def insert_text(session: Session, content: Content) -> int:
    """Insert one ``texts`` row; return its ``id``."""
    fmt, cnt = _encode_content(content)
    row = TextRow(formatting=fmt, content=cnt)
    session.add(row)
    session.flush()
    return row.id  # type: ignore[return-value]


def insert_paragraph(session: Session, content: Content) -> int:
    """
    Insert a ``paragraphs`` row containing a single Text element built from
    *content*.  Returns the paragraph ``id``.
    """
    text_id = insert_text(session, content)

    elem = Element(kind=ElementKindEnum.text, text_id=text_id)
    session.add(elem)
    session.flush()

    para = Paragraph()
    session.add(para)
    session.flush()

    pe = ParagraphElement(paragraph_id=para.id, position=0, element_id=elem.id)
    session.add(pe)

    return para.id  # type: ignore[return-value]


def insert_choices(session: Session, choices: list[Content]) -> int:
    """
    Insert a ``lists`` row of multiple-choice options (lowercase alphabetic,
    right-parenthesis format).  Returns the list ``id``.
    """
    lst = ListRow(
        order_type=OrderTypeEnum.lowercase_alphabetic,
        order_format=OrderFormatEnum.right_parenthesis,
    )
    session.add(lst)
    session.flush()

    for pos, choice in enumerate(choices):
        para_id = insert_paragraph(session, choice)
        session.add(ListItem(list_id=lst.id, position=pos, paragraph_id=para_id))

    return lst.id  # type: ignore[return-value]


def insert_elemental_question(
    session: Session,
    q: ExtractedQuestion,
    id_prefix: str = "",
    _used_ids: set[str] | None = None,
) -> str:
    """Insert an ``elemental_questions`` row; return its string ``id``.

    If the desired id already exists in the DB or in *_used_ids* (within the
    current transaction), a numeric suffix is appended until a free slot is
    found (e.g. ``p1-2`` → ``p1-2-2`` → ``p1-2-3``).
    """
    base_id = f"{id_prefix}{q.id}" if id_prefix else q.id

    # Determine a unique id
    candidate = base_id
    counter = 2
    while True:
        # Check set of ids already inserted in this batch
        if _used_ids is not None and candidate in _used_ids:
            candidate = f"{base_id}-{counter}"
            counter += 1
            continue
        # Check whether it already exists in the DB
        if session.get(ElementalQuestion, candidate) is not None:
            candidate = f"{base_id}-{counter}"
            counter += 1
            continue
        break

    if _used_ids is not None:
        _used_ids.add(candidate)
    content_para_id = insert_paragraph(session, q.content)

    answer_para_id = insert_paragraph(session, q.answer) if q.answer else None
    solution_para_id = insert_paragraph(session, q.solution) if q.solution else None
    choice_pool_id = insert_choices(session, q.choices) if q.choices else None

    block_kind = QuestionBlockKindEnum(q.block_type)
    block_lines = q.block_lines
    block_space = q.block_space

    # Normalise: enforce DB check constraints so we never hit a CHECK violation.
    # essay requires block_lines; solve/proof require block_space.
    if block_kind == QuestionBlockKindEnum.essay and block_lines is None:
        if block_space is not None:
            # LLM gave space instead of lines — treat as 'solve'
            block_kind = QuestionBlockKindEnum.solve
            print(
                f"[store] WARNING: question {candidate!r} had block_kind='essay' "
                f"but only block_space={block_space}; converted to 'solve'."
            )
        else:
            block_kind = QuestionBlockKindEnum.none
            print(
                f"[store] WARNING: question {candidate!r} had block_kind='essay' "
                "with no block_lines or block_space; converted to 'none'."
            )
    elif block_kind in (QuestionBlockKindEnum.solve, QuestionBlockKindEnum.proof) and block_space is None:
        original_kind = block_kind.value
        block_kind = QuestionBlockKindEnum.none
        print(
            f"[store] WARNING: question {candidate!r} had block_kind={original_kind!r} "
            "but block_space was None; converted to 'none'."
        )

    eq = ElementalQuestion(
        id=candidate,
        content_id=content_para_id,
        answer_id=answer_para_id,
        solution_id=solution_para_id,
        choice_pool_id=choice_pool_id,
        block_kind=block_kind,
        block_lines=block_lines,
        block_space=block_space,
    )
    session.add(eq)
    session.flush()
    return eq.id  # type: ignore[return-value]


def insert_elemental_problem(
    session: Session,
    p: ExtractedProblem,
    id_prefix: str = "",
    _used_ids: set[str] | None = None,
) -> int:
    """
    Insert an ``elemental_problems`` row (and all referenced rows).
    Returns the ``elemental_problems.id``.
    """
    if p.kind == "question":
        if p.question is None:
            raise ValueError("ExtractedProblem.kind='question' but .question is None")
        q_id = insert_elemental_question(session, p.question, id_prefix, _used_ids)
        ep = ElementalProblem(kind=ElementalProblemKindEnum.question, question_id=q_id)

    elif p.kind == "series":
        if p.series is None:
            raise ValueError("ExtractedProblem.kind='series' but .series is None")
        intro_para_id = insert_paragraph(session, p.series.intro)
        qs = QuestionSeries(
            content_id=intro_para_id,
            order_type=OrderTypeEnum.decimal,
            order_format=OrderFormatEnum.period,
        )
        session.add(qs)
        session.flush()

        for pos, sub_q in enumerate(p.series.questions):
            sub_id = insert_elemental_question(session, sub_q, id_prefix, _used_ids)
            session.add(
                QuestionSeriesItem(series_id=qs.id, position=pos, question_id=sub_id)
            )

        ep = ElementalProblem(kind=ElementalProblemKindEnum.block, series_id=qs.id)

    elif p.kind == "plain":
        if p.plain is None:
            raise ValueError("ExtractedProblem.kind='plain' but .plain is None")
        para_id = insert_paragraph(session, p.plain)
        ep = ElementalProblem(kind=ElementalProblemKindEnum.plain, paragraph_id=para_id)

    else:
        raise ValueError(f"Unknown ExtractedProblem.kind: {p.kind!r}")

    session.add(ep)
    session.flush()
    return ep.id  # type: ignore[return-value]


# ══════════════════════════════════════════════════════════════════════════════
# Top-level store function
# ══════════════════════════════════════════════════════════════════════════════


def store_page(
    session: Session,
    page: ExtractedPage,
    category_id: int,
    id_prefix: str = "",
) -> list[int]:
    """
    Persist all problems from one LLM-extracted page as ``single_problems``.

    Args:
        session:     Active SQLAlchemy session (caller manages commit/rollback).
        page:        LLM-extracted :class:`~schema.ExtractedPage`.
        category_id: FK into ``problem_categories`` (must already exist).
        id_prefix:   Prepended to every question ``id`` to avoid collisions
                     across pages / files (e.g. ``"doc1-p3-"``).

    Returns:
        List of ``single_problems.id`` values created.
    """
    cat = session.get(ProblemCategory, category_id)
    if cat is None:
        raise LookupError(f"No problem_categories row with id={category_id}")

    sp_ids: list[int] = []
    used_ids: set[str] = set()
    for prob in page.problems:
        ep_id = insert_elemental_problem(session, prob, id_prefix, _used_ids=used_ids)
        sp = SingleProblem(problem_id=ep_id, category_id=category_id)
        session.add(sp)
        session.flush()
        sp_ids.append(sp.id)  # type: ignore[arg-type]

    return sp_ids
