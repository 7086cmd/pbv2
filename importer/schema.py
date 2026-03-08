"""
Pydantic models representing the LLM-extracted structure of a single exam page.

These map to the DB schema (defined in db_models.py) and are the contract
between the LLM and the storage layer.
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


# ── Questions ─────────────────────────────────────────────────────────────────


class ExtractedQuestion(BaseModel):
    """One atomic question extracted from the OCR output."""

    id: str = Field(
        description="Short question identifier matching print numbering, e.g. '1', '2a', '3ii'."
    )
    content: Content = Field(description="The question stem / body.")
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


class ExtractedSeries(BaseModel):
    """A block question: shared intro paragraph + list of numbered sub-questions."""

    intro: Content = Field(description="The shared context / stimulus paragraph.")
    questions: list[ExtractedQuestion] = Field(
        description="Ordered sub-questions belonging to this series."
    )


# ── Top-level problem ─────────────────────────────────────────────────────────


class ExtractedProblem(BaseModel):
    """
    One problem found on the page.  Exactly one of ``question``, ``series``,
    or ``plain`` should be populated, matching ``kind``.
    """

    kind: Literal["question", "series", "plain"] = Field(
        description=(
            "'question' = standalone question; "
            "'series' = group with intro + sub-questions; "
            "'plain' = non-question text (heading, instruction, etc.)."
        )
    )
    question: Optional[ExtractedQuestion] = None
    series: Optional[ExtractedSeries] = None
    plain: Optional[Content] = None


# ── Page result ───────────────────────────────────────────────────────────────


class ExtractedPage(BaseModel):
    """Full LLM output for one OCR-processed page or chunk."""

    problems: list[ExtractedProblem] = Field(
        description="All problems found on this page, in reading order."
    )
