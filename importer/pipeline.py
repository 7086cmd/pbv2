"""
Import pipeline orchestration.

PDF flow
──────────
  1. Render each page → PNG bytes                 (PyMuPDF)
  2. OCR each PNG                                  (Baidu doc_analysis)
  3. Group OCR text into chunks of N pages
  4. Call the LLM once per chunk
  5. Persist to database via store layer

Document flow (Word / TeX / Markdown)
──────────────────────────────────────
  1. Extract plain text + LaTeX math from the file (pypandoc / direct read)
  2. Split into chunks of at most M characters
  3. Call the LLM once per chunk
  4. Persist to database via store layer

The two flows share the same LLM + store loop (_run_chunks).
"""

from __future__ import annotations

import json
import os
from pathlib import Path
from typing import Callable

import fitz  # PyMuPDF
from sqlalchemy import create_engine
from sqlalchemy.orm import Session

from doc_extractor import chunk_text, extract_document
from llm import LLMBackend, get_backend
from ocr import extract_text_and_figures, ocr_image
from store import store_page


# ── PDF rendering ─────────────────────────────────────────────────────────────


def render_pdf_pages(pdf_path: Path, dpi: int = 200) -> list[bytes]:
    """Render every page of *pdf_path* to a PNG at *dpi* dots-per-inch."""
    doc = fitz.open(str(pdf_path))
    scale = dpi / 72.0
    mat = fitz.Matrix(scale, scale)
    pages: list[bytes] = []
    for page in doc:
        pix = page.get_pixmap(matrix=mat, alpha=False)
        pages.append(pix.tobytes("png"))
    return pages


# ── System prompt ─────────────────────────────────────────────────────────────


def load_system_prompt() -> str:
    """Return the contents of ``prompt.txt`` next to this file, or a sensible default."""
    prompt_path = Path(__file__).parent / "prompt.txt"
    if prompt_path.exists():
        text = prompt_path.read_text(encoding="utf-8").strip()
        if text:
            return text
    return (
        "You are an expert educational content extractor. "
        "Extract all problems from the provided OCR text and return structured JSON."
    )


# ── Shared LLM + store loop ──────────────────────────────────────────────────


def _run_chunks(
    chunks: list[tuple[str, str]],
    *,
    category_id: int | None,
    engine,
    backend: LLMBackend,
    system_prompt: str,
    verbose: bool,
    dry_run: bool,
    log: Callable[[str], None],
) -> list[int]:
    """
    Run LLM extraction + database storage for a list of ``(id_prefix, text)``
    chunks.  Each chunk is one LLM call; results are committed per-chunk with
    rollback on error.

    Returns the list of ``single_problems.id`` values created (empty for dry-run).
    """
    all_sp_ids: list[int] = []

    if dry_run:
        for chunk_prefix, chunk_text_item in chunks:
            log(f"[pipeline] LLM call (prefix={chunk_prefix!r}, {len(chunk_text_item)} chars) …")
            try:
                extracted = backend.extract(system_prompt, chunk_text_item)
            except Exception as exc:
                log(f"[pipeline]   LLM failed: {exc}")
                continue
            log(f"[pipeline]   {len(extracted.problems)} problem(s):")
            print(json.dumps(extracted.model_dump(), ensure_ascii=False, indent=2))
        log("[pipeline] Dry run complete.")
        return []

    with Session(engine) as session:
        for chunk_prefix, chunk_text_item in chunks:
            log(f"[pipeline] LLM call (prefix={chunk_prefix!r}, {len(chunk_text_item)} chars) …")
            try:
                extracted = backend.extract(system_prompt, chunk_text_item)
            except Exception as exc:
                log(f"[pipeline]   LLM failed: {exc}  – skipping chunk")
                continue

            log(f"[pipeline]   {len(extracted.problems)} problem(s). Storing …")
            try:
                sp_ids = store_page(session, extracted, category_id, id_prefix=chunk_prefix)
                session.commit()
            except Exception as exc:
                session.rollback()
                log(f"[pipeline]   DB store failed: {exc}  – rolling back chunk")
                continue

            log(f"[pipeline]   Stored → single_problem ids: {sp_ids}")
            all_sp_ids.extend(sp_ids)

    return all_sp_ids


# ── PDF pipeline ───────────────────────────────────────────────────────────────


def run_pipeline(
    pdf_path: Path,
    category_id: int | None,
    *,
    id_prefix: str = "",
    dpi: int = 200,
    max_pages: int | None = None,
    per_page: bool = False,
    chunk_pages: int = 3,
    backend: LLMBackend | None = None,
    verbose: bool = True,
    dry_run: bool = False,
) -> list[int]:
    """
    Run the full import pipeline for one PDF file.

    Pages are grouped into batches of *chunk_pages* (default 3).  Each batch
    is OCR'd and sent to the LLM as one call so questions that straddle a
    page boundary are handled correctly, while keeping outputs within the
    model's token budget.  Set *per_page=True* to call the LLM once per page
    (legacy behaviour, not recommended).

    Args:
        pdf_path:     Path to the PDF to import.
        category_id:  ``problem_categories.id`` for all imported problems.
                      Required unless *dry_run* is True.
        id_prefix:    Prefix prepended to every question id.
        dpi:          Rasterisation DPI for the PDF→PNG step.
        max_pages:    Cap the number of pages processed.
        per_page:     If True, call the LLM once per page (ignores chunk_pages).
        chunk_pages:  Pages per LLM call in whole-document (default) mode.
        backend:      Pre-built LLM backend; created from env vars if None.
        verbose:      Print progress to stdout.
        dry_run:      Skip DB writes; print extracted JSON instead.

    Returns:
        List of ``single_problems.id`` values created (empty for dry_run).
    """
    if not dry_run:
        db_url = os.environ["DATABASE_URL"]
        engine = create_engine(db_url)
    else:
        engine = None  # type: ignore[assignment]

    if backend is None:
        backend = get_backend()

    system_prompt = load_system_prompt()

    def log(msg: str) -> None:
        if verbose:
            print(msg)

    log(f"[pipeline] Rendering PDF: {pdf_path}")
    pages = render_pdf_pages(pdf_path, dpi=dpi)
    if max_pages is not None:
        pages = pages[:max_pages]
    log(f"[pipeline] {len(pages)} page(s) to process at {dpi} DPI")

    # ── OCR all pages ──────────────────────────────────────────────────────────
    log("[pipeline] Running OCR on all pages …")
    page_texts: list[str] = []
    for page_no, png_bytes in enumerate(pages, start=1):
        log(f"[pipeline]   OCR page {page_no}/{len(pages)} …")
        try:
            ocr_result = ocr_image(png_bytes)
        except Exception as exc:
            log(f"[pipeline]   OCR failed on page {page_no}: {exc}  – inserting blank")
            page_texts.append("")
            continue
        text, _figures = extract_text_and_figures(ocr_result, png_bytes)
        page_texts.append(text)
        log(f"[pipeline]   Page {page_no}: {len(text)} chars")

    all_sp_ids: list[int] = []

    if per_page:
        # ── Per-page mode (legacy) ─────────────────────────────────────────────
        chunks: list[tuple[str, str]] = [
            (f"{id_prefix}p{i+1}-", t)
            for i, t in enumerate(page_texts)
            if t.strip()
        ]
    else:
        # ── Chunked mode (default) ─────────────────────────────────────────────
        # Group consecutive pages into batches of chunk_pages so that questions
        # straddling a page boundary stay in the same LLM call, while keeping
        # output size predictable.
        chunks = []
        for batch_start in range(0, len(page_texts), chunk_pages):
            batch = page_texts[batch_start : batch_start + chunk_pages]
            combined = "\n\n".join(
                f"── Page {batch_start + j + 1} ──\n{t}"
                for j, t in enumerate(batch)
                if t.strip()
            )
            if combined.strip():
                chunk_prefix = f"{id_prefix}p{batch_start + 1}-"
                chunks.append((chunk_prefix, combined))

    if not chunks:
        log("[pipeline] No text extracted from any page.")
        return []

    log(
        f"[pipeline] Calling LLM {'per page' if per_page else f'in chunks of {chunk_pages} pages'} "
        f"({len(chunks)} call(s)) …"
    )

    all_sp_ids = _run_chunks(
        chunks,
        category_id=category_id,
        engine=engine,
        backend=backend,
        system_prompt=system_prompt,
        verbose=verbose,
        dry_run=dry_run,
        log=log,
    )

    log(f"[pipeline] Done. {len(all_sp_ids)} problem(s) imported total.")
    return all_sp_ids


# ── Document pipeline (Word / TeX / Markdown) ─────────────────────────────────


def run_doc_pipeline(
    doc_path: Path,
    category_id: int | None,
    *,
    id_prefix: str = "",
    chunk_chars: int = 6000,
    backend: LLMBackend | None = None,
    verbose: bool = True,
    dry_run: bool = False,
) -> list[int]:
    """
    Import a Word / TeX / Markdown exam document into the database.

    Text + LaTeX math is extracted from the file without OCR, then chunked
    and sent to the LLM using the same system prompt as the PDF pipeline.

    Args:
        doc_path:    Path to a ``.docx``, ``.tex``, or ``.md`` file.
        category_id: ``problem_categories.id`` to tag all problems with.
                     Required unless *dry_run* is True.
        id_prefix:   Prefix prepended to every question id.
        chunk_chars: Maximum characters per LLM call.
        backend:     Pre-built LLM backend; created from env vars if None.
        verbose:     Print progress to stdout.
        dry_run:     Skip DB writes; print extracted JSON instead.

    Returns:
        List of ``single_problems.id`` values created (empty for dry_run).
    """
    if not dry_run:
        db_url = os.environ["DATABASE_URL"]
        engine = create_engine(db_url)
    else:
        engine = None  # type: ignore[assignment]

    if backend is None:
        backend = get_backend()

    system_prompt = load_system_prompt()

    def log(msg: str) -> None:
        if verbose:
            print(msg)

    # ── Extract text ───────────────────────────────────────────────────────────
    log(f"[pipeline] Extracting text from: {doc_path}")
    try:
        full_text = extract_document(doc_path)
    except Exception as exc:
        raise RuntimeError(f"Failed to extract text from {doc_path}: {exc}") from exc

    log(f"[pipeline] {len(full_text)} chars extracted from document.")

    # ── Chunk ──────────────────────────────────────────────────────────────────
    text_chunks = chunk_text(full_text, chunk_chars=chunk_chars)
    log(f"[pipeline] Split into {len(text_chunks)} chunk(s) of ≤{chunk_chars} chars each.")

    chunks: list[tuple[str, str]] = [
        (f"{id_prefix}pt{i + 1}-", t)
        for i, t in enumerate(text_chunks)
    ]

    log(f"[pipeline] Calling LLM ({len(chunks)} call(s)) …")

    all_sp_ids = _run_chunks(
        chunks,
        category_id=category_id,
        engine=engine,
        backend=backend,
        system_prompt=system_prompt,
        verbose=verbose,
        dry_run=dry_run,
        log=log,
    )

    log(f"[pipeline] Done. {len(all_sp_ids)} problem(s) imported total.")
    return all_sp_ids
