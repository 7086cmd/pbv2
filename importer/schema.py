"""
Pydantic models representing the LLM-extracted structure of a single exam page.

These map to the DB schema (defined in db_models.py) and are the contract
between the LLM and the storage layer.

Schema conventions
──────────────────
Every problem is an ElementalQuestion.  Sub-questions (e.g. "(a) … (b) …")
are expressed as ``sub_questions``: a flat ordered list of Content items that
the store layer writes as a single List element appended to the content
paragraph.  There is no separate "series" wrapper; the introductory stem lives
in ``content`` and the numbered parts live in ``sub_questions``.
"""

from __future__ import annotations

from typing import Literal, Optional

from pydantic import BaseModel, Field


# ── Text runs ─────────────────────────────────────────────────────────────────


class TextRun(BaseModel):
    """A single homogeneous run of text or an inline LaTeX formula."""

    text: str = Field(description="The text content; LaTeX source when is_formula=true.")
    is_formula: bool = Field(
        default=False,
        description="True when this run is a LaTeX math expression (inline or display).",
    )
    bold: bool = Field(default=False)
    italic: bool = Field(default=False)


class Content(BaseModel):
    """Ordered sequence of TextRuns forming a paragraph or content block."""

    runs: list[TextRun]

    def plain(self) -> str:
        """Return a human-readable representation with formulas wrapped in $…$."""
        parts: list[str] = []
        for r in self.runs:
            parts.append(f"${r.text}$" if r.is_formula else r.text)
        return "".join(parts)


# ── Sub-questions ──────────────────────────────────────────────────────────────


class SubQuestion(BaseModel):
    """
    One part of a multi-part question.

    Stored as a List element inside the parent ElementalQuestion's content
    paragraph — not as a separate DB entity.
    """

    content: Content = Field(description="The text of this sub-question part.")


# ── Question ───────────────────────────────────────────────────────────────────


class ExtractedQuestion(BaseModel):
    """One atomic question (or multi-part question) extracted from the OCR output."""

    id: str = Field(
        description="Short question identifier matching print numbering, e.g. '1', '2', '3'."
    )
    content: Content = Field(
        description=(
            "The question stem / introductory text. "
            "For multi-part questions this is the shared stimulus; "
            "the individual parts go in sub_questions."
        )
    )
    sub_questions: Optional[list[SubQuestion]] = Field(
        default=None,
        description=(
            "Ordered sub-question parts (a), (b), (c) … "
            "Present only when the question explicitly breaks into labelled parts. "
            "Each part is stored as a list item in the content paragraph."
        ),
    )
    answer: Optional[Content] = Field(
        default=None,
        description="Printed answer, if present (e.g. answer key printed alongside).",
    )
    solution: Optional[Content] = Field(
        default=None,
        description="Printed worked solution, if present.",
    )
    choices: Optional[list[Content]] = Field(
        default=None,
        description="Multiple-choice options in order, if this is an MCQ.",
    )
    block_type: Literal["essay", "proof", "solve", "none"] = Field(
        default="none",
        description=(
            "Answer-space type on the question sheet. "
            "'essay' = ruled lines; 'proof'/'solve' = blank space; 'none' = no space."
        ),
    )
    block_lines: Optional[int] = Field(
        default=None,
        description="For block_type='essay': number of ruled answer lines.",
    )
    block_space: Optional[float] = Field(
        default=None,
        description="For block_type='proof' or 'solve': space in rem.",
    )


# ── Page result ───────────────────────────────────────────────────────────────


class ExtractedPage(BaseModel):
    """Full LLM output for one OCR-processed page or chunk."""

    problems: list[ExtractedQuestion] = Field(
        description="All questions found on this page, in reading order."
    )
