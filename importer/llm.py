"""
LLM backends for structured extraction.

Supported providers (set via LLM_PROVIDER env var):
  OPENAI   – ChatGPT via openai SDK  (default)
  gemini  – Gemini via google-genai SDK

Environment variables:
  LLM_PROVIDER   openai | gemini           (default: openai)
  CHATGPT_API_KEY                          (required for openai)
  OPENAI_MODEL   e.g. gpt-4o-2024-08-06   (default: gpt-4o-2024-08-06)
  GEMINI_API_KEY                           (required for gemini)
  GEMINI_MODEL   e.g. gemini-2.5-pro-...  (default: gemini-2.5-pro-preview-03-25)

Optional:
  OPENAI_MAX_OUTPUT_TOKENS  (default: 16384)
  GEMINI_MAX_OUTPUT_TOKENS  (default: 16384)
"""

from __future__ import annotations

import os
from abc import ABC, abstractmethod

from schema import ExtractedPage


# ── Abstract base ─────────────────────────────────────────────────────────────


class LLMBackend(ABC):
    @abstractmethod
    def extract(self, system_prompt: str, ocr_text: str) -> ExtractedPage:
        """
        Call the LLM with the given system prompt and OCR text.
        Returns a validated :class:`ExtractedPage` instance.
        """
        ...


# ── OpenAI (ChatGPT) ──────────────────────────────────────────────────────────


class OpenAIBackend(LLMBackend):
    """Uses Chat Completions beta parse endpoint with Structured Outputs (Pydantic)."""

    def __init__(self) -> None:
        from openai import OpenAI

        self._client = OpenAI(api_key=os.environ["CHATGPT_API_KEY"])
        self._model = os.environ.get("OPENAI_MODEL", "gpt-4o-2024-08-06")
        self._max_tokens = int(os.environ.get("OPENAI_MAX_OUTPUT_TOKENS", "16384"))

    def extract(self, system_prompt: str, ocr_text: str) -> ExtractedPage:
        completion = self._client.beta.chat.completions.parse(
            model=self._model,
            max_tokens=self._max_tokens,
            messages=[
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": ocr_text},
            ],
            response_format=ExtractedPage,
        )
        choice = completion.choices[0]
        msg = choice.message
        finish = choice.finish_reason
        usage = completion.usage
        print(
            f"[llm] finish_reason={finish!r}  "
            f"prompt_tokens={usage.prompt_tokens if usage else '?'}  "
            f"completion_tokens={usage.completion_tokens if usage else '?'}"
        )
        if finish == "length":
            print("[llm] WARNING: output was truncated (max_tokens limit hit)!")
        if msg.refusal:
            raise RuntimeError(f"OpenAI refusal: {msg.refusal!r}")
        if msg.parsed is None:
            raise RuntimeError("OpenAI returned no parsed output.")
        return msg.parsed


# ── Gemini ────────────────────────────────────────────────────────────────────


class GeminiBackend(LLMBackend):
    """Uses google-genai with JSON schema constrained generation."""

    def __init__(self) -> None:
        from google import genai

        self._client = genai.Client(api_key=os.environ["GEMINI_API_KEY"])
        self._model = os.environ.get("GEMINI_MODEL", "gemini-2.5-pro-preview-03-25")
        self._max_tokens = int(os.environ.get("GEMINI_MAX_OUTPUT_TOKENS", "16384"))

    def extract(self, system_prompt: str, ocr_text: str) -> ExtractedPage:
        prompt = f"{system_prompt}\n\n---\n\n{ocr_text}"
        response = self._client.models.generate_content(
            model=self._model,
            contents=prompt,
            config={
                "response_mime_type": "application/json",
                "response_json_schema": ExtractedPage.model_json_schema(),
                "max_output_tokens": self._max_tokens,
            },
        )
        return ExtractedPage.model_validate_json(response.text)


# ── Factory ───────────────────────────────────────────────────────────────────


def get_backend() -> LLMBackend:
    """Return the configured LLM backend based on the LLM_PROVIDER env var."""
    provider = os.environ.get("LLM_PROVIDER", "openai").lower()
    if provider == "openai":
        return OpenAIBackend()
    if provider == "gemini":
        return GeminiBackend()
    raise ValueError(
        f"Unknown LLM_PROVIDER={provider!r}. Valid values: 'openai', 'gemini'."
    )
