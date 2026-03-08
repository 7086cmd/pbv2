"""
Extract plain text (with LaTeX math) from Word, TeX, and Markdown documents.

Supported formats
─────────────────
  .docx   Microsoft Word  (OMML math → LaTeX via pandoc)
  .tex    LaTeX source    (preamble stripped; math preserved verbatim)
  .md     Markdown        (read as-is; $…$ / $$…$$ math preserved)

MathType note
─────────────
MathType equations stored as OLE objects in .docx files cannot be extracted
automatically.  Before importing, run MathType → "Convert Equations" →
"MathType 6 / MS Office" → "Equations to Microsoft Office Equations (OMML)"
to convert them to native Word math, which pandoc can handle.

Pandoc requirement
──────────────────
The Word (docx) extractor requires pandoc to be installed on the system.
  macOS:   brew install pandoc
  Linux:   apt install pandoc
  Windows: https://pandoc.org/installing.html
"""

from __future__ import annotations

import re
from pathlib import Path


# ── Per-format extractors ──────────────────────────────────────────────────────


def extract_docx(path: Path) -> str:
    """Convert a Word document to Markdown text, with OMML math as LaTeX.

    Requires `pypandoc` (pip/uv) and a system-installed `pandoc` binary.
    Math is output as inline ``$...$`` / display ``$$...$$`` LaTeX.
    """
    try:
        import pypandoc  # type: ignore[import-untyped]
    except ImportError as exc:
        raise RuntimeError(
            "pypandoc is required for Word import.\n"
            "  Install it with:  uv add pypandoc\n"
            "  Also requires pandoc: https://pandoc.org/installing.html"
        ) from exc

    text: str = pypandoc.convert_file(
        str(path),
        to="markdown",
        format="docx",
        extra_args=[
            "--wrap=none",          # no hard line-wraps
            "--markdown-headings=atx",  # ATX-style headings  (## …)
        ],
    )
    return text.strip()


def extract_tex(path: Path) -> str:
    """Return the body of a LaTeX document.

    Extracts text between ``\\begin{document}`` and ``\\end{document}``.
    Falls back to the whole file if no document environment is found.
    Math is already in LaTeX syntax so no conversion is needed.
    """
    raw = path.read_text(encoding="utf-8", errors="replace")

    m = re.search(
        r"\\begin\{document\}(.*?)\\end\{document\}",
        raw,
        flags=re.DOTALL,
    )
    if m:
        return m.group(1).strip()
    return raw.strip()


def extract_markdown(path: Path) -> str:
    """Read a Markdown file as-is, preserving ``$...$`` and ``$$...$$`` math."""
    return path.read_text(encoding="utf-8", errors="replace").strip()


# ── Dispatcher ────────────────────────────────────────────────────────────────


#: Map of supported file suffixes to human-readable format names.
SUPPORTED_SUFFIXES: dict[str, str] = {
    ".docx": "Microsoft Word",
    ".tex":  "LaTeX",
    ".latex": "LaTeX",
    ".md":   "Markdown",
    ".markdown": "Markdown",
}


def extract_document(path: Path) -> str:
    """Extract text from *path*, dispatching on its file suffix.

    Args:
        path: Path to a ``.docx``, ``.tex``/``.latex``, or ``.md``/``.markdown``
              file.

    Returns:
        Plain text (with LaTeX math) ready to be sent to the LLM.

    Raises:
        ValueError: If the file format is not supported or is ``.doc``.
    """
    suffix = path.suffix.lower()

    if suffix == ".doc":
        raise ValueError(
            "Legacy .doc (Word 97–2003) format is not supported.\n"
            "Open in Microsoft Word and save as .docx, then retry."
        )

    if suffix in (".docx",):
        return extract_docx(path)
    if suffix in (".tex", ".latex"):
        return extract_tex(path)
    if suffix in (".md", ".markdown"):
        return extract_markdown(path)

    raise ValueError(
        f"Unsupported file format {suffix!r}.\n"
        f"Supported formats: {', '.join(SUPPORTED_SUFFIXES)}"
    )


# ── Text chunker ──────────────────────────────────────────────────────────────


def chunk_text(text: str, chunk_chars: int = 6000) -> list[str]:
    """Split *text* into chunks of at most *chunk_chars* characters.

    Tries to break at paragraph boundaries (blank lines) to avoid splitting
    a question mid-sentence.  If a single paragraph exceeds *chunk_chars* it
    is hard-cut at *chunk_chars*.

    Args:
        text:        The full document text to split.
        chunk_chars: Maximum characters per chunk.

    Returns:
        List of non-empty text chunks.
    """
    if len(text) <= chunk_chars:
        return [text]

    paragraphs = re.split(r"\n{2,}", text)
    chunks: list[str] = []
    current = ""

    for para in paragraphs:
        candidate = (current + "\n\n" + para).lstrip("\n") if current else para

        if len(candidate) <= chunk_chars:
            current = candidate
        else:
            # Flush what we have so far
            if current:
                chunks.append(current)
            # Hard-split if a single paragraph is too large
            if len(para) > chunk_chars:
                for i in range(0, len(para), chunk_chars):
                    chunks.append(para[i : i + chunk_chars])
                current = ""
            else:
                current = para

    if current:
        chunks.append(current)

    return [c for c in chunks if c.strip()]
