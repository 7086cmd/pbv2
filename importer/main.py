"""
CLI entry point for the PBv2 importer.

Usage
─────
  python main.py import-pdf  <PDF>  --category-id <ID> [options]
  python main.py import-doc  <DOC>  --category-id <ID> [options]
  python main.py import-pdf  <PDF>  --dry-run   # skip DB, print JSON
  python main.py import-doc  <DOC>  --dry-run

Supported document formats for import-doc
─────────────────────────────────────────
  .docx   Microsoft Word (OMML / native Word math → LaTeX via pandoc)
  .tex    LaTeX source (preamble stripped automatically)
  .md     Markdown

Required environment variables (or set in .env next to this file)
───────────────────────────────────────────────────────────────────
  DATABASE_URL         PostgreSQL connection string (not needed for --dry-run)
                       e.g. postgresql://user:pass@localhost/pbv2

  BAIDU_API_KEY        Baidu AI Platform API key (OCR step, PDF only)
  BAIDU_SECRET_KEY     Baidu AI Platform secret key (OCR step, PDF only)

  LLM_PROVIDER         'openai' (default) or 'gemini'

  CHATGPT_API_KEY      Required when LLM_PROVIDER=openai
  GEMINI_API_KEY       Required when LLM_PROVIDER=gemini

Optional environment variables
────────────────────────────────
  OPENAI_MODEL         Default: gpt-4o-2024-08-06
  GEMINI_MODEL         Default: gemini-2.5-pro-preview-03-25
"""

from __future__ import annotations

from pathlib import Path

import click
from dotenv import load_dotenv

# Load .env from the importer directory automatically
load_dotenv(Path(__file__).parent / ".env")

from pipeline import run_doc_pipeline, run_pipeline  # noqa: E402 (after dotenv)


@click.group()
def cli() -> None:
    """PBv2 importer – ingest exam papers into the database.

    Supported input formats: PDF (.pdf), Word (.docx), LaTeX (.tex), Markdown (.md).
    """


@cli.command("import-pdf")
@click.argument("pdf_path", type=click.Path(exists=True, path_type=Path))
@click.option(
    "--category-id",
    "-c",
    default=None,
    type=int,
    help="problem_categories.id to tag all imported problems with.",
)
@click.option(
    "--prefix",
    "-p",
    default="",
    show_default=True,
    help="Prefix for question IDs to avoid collisions (e.g. 'hw01-').",
)
@click.option(
    "--dpi",
    default=200,
    show_default=True,
    type=int,
    help="DPI for PDF-to-PNG rasterisation.",
)
@click.option(
    "--max-pages",
    default=None,
    type=int,
    help="Limit processing to the first N pages (useful for testing).",
)
@click.option(
    "--chunk-pages",
    default=3,
    show_default=True,
    type=int,
    help="Pages per LLM call (ignored when --per-page is set).",
)
@click.option(
    "--per-page",
    is_flag=True,
    default=False,
    help="Call the LLM once per page instead of once for the whole document.",
)
@click.option(
    "--dry-run",
    is_flag=True,
    default=False,
    help="Run OCR + LLM but skip database writes; print extracted JSON instead.",
)
@click.option(
    "--quiet",
    "-q",
    is_flag=True,
    default=False,
    help="Suppress progress output.",
)
def import_pdf(
    pdf_path: Path,
    category_id: int | None,
    prefix: str,
    dpi: int,
    max_pages: int | None,
    chunk_pages: int,
    per_page: bool,
    dry_run: bool,
    quiet: bool,
) -> None:
    """Import a PDF exam paper and store problems in the database."""
    if not dry_run and category_id is None:
        raise click.UsageError("--category-id is required unless --dry-run is set.")

    sp_ids = run_pipeline(
        pdf_path,
        category_id=category_id,
        id_prefix=prefix,
        dpi=dpi,
        max_pages=max_pages,
        chunk_pages=chunk_pages,
        per_page=per_page,
        verbose=not quiet,
        dry_run=dry_run,
    )
    if not dry_run:
        click.echo(f"Imported {len(sp_ids)} problem(s).")


@cli.command("import-doc")
@click.argument("doc_path", type=click.Path(exists=True, path_type=Path))
@click.option(
    "--category-id",
    "-c",
    default=None,
    type=int,
    help="problem_categories.id to tag all imported problems with.",
)
@click.option(
    "--prefix",
    "-p",
    default="",
    show_default=True,
    help="Prefix for question IDs to avoid collisions (e.g. 'hw01-').",
)
@click.option(
    "--chunk-chars",
    default=6000,
    show_default=True,
    type=int,
    help="Maximum characters per LLM call when splitting large documents.",
)
@click.option(
    "--dry-run",
    is_flag=True,
    default=False,
    help="Extract text + LLM but skip database writes; print extracted JSON instead.",
)
@click.option(
    "--quiet",
    "-q",
    is_flag=True,
    default=False,
    help="Suppress progress output.",
)
def import_doc(
    doc_path: Path,
    category_id: int | None,
    prefix: str,
    chunk_chars: int,
    dry_run: bool,
    quiet: bool,
) -> None:
    """Import a Word (.docx), LaTeX (.tex), or Markdown (.md) exam paper.

    Text and formulas are extracted directly from the file (no OCR).
    Word documents use pandoc to convert OMML equations to LaTeX;
    MathType equations stored as OLE objects must be converted to OMML
    first using MathType \u2192 \"Convert Equations\" \u2192 \"Office 2007 or later (OMML)\".
    """
    if not dry_run and category_id is None:
        raise click.UsageError("--category-id is required unless --dry-run is set.")

    sp_ids = run_doc_pipeline(
        doc_path,
        category_id=category_id,
        id_prefix=prefix,
        chunk_chars=chunk_chars,
        verbose=not quiet,
        dry_run=dry_run,
    )
    if not dry_run:
        click.echo(f"Imported {len(sp_ids)} problem(s).")


if __name__ == "__main__":
    cli()
