"""
Baidu OCR (doc_analysis) wrapper.

Environment variables required:
  BAIDU_OCR_API_KEY    – Baidu AI Platform API key
  BAIDU_OCR_SECRET_KEY – Baidu AI Platform secret key
"""

from __future__ import annotations

import base64
import io
import os
from typing import TYPE_CHECKING

import httpx

if TYPE_CHECKING:
    pass

BAIDU_OCR_URL = "https://aip.baidubce.com/rest/2.0/ocr/v1/doc_analysis"
BAIDU_TOKEN_URL = "https://aip.baidubce.com/oauth/2.0/token"


# ── Auth ──────────────────────────────────────────────────────────────────────


def _get_access_token(api_key: str, secret_key: str) -> str:
    resp = httpx.post(
        BAIDU_TOKEN_URL,
        params={
            "grant_type": "client_credentials",
            "client_id": api_key,
            "client_secret": secret_key,
        },
        timeout=30,
    )
    resp.raise_for_status()
    data = resp.json()
    if "access_token" not in data:
        raise RuntimeError(f"Baidu token error: {data}")
    return data["access_token"]


# ── Core OCR call ─────────────────────────────────────────────────────────────


def ocr_image(png_bytes: bytes) -> dict:
    """
    Run Baidu doc_analysis OCR on a PNG image.

    Returns the full JSON response dict, which includes:
      - results / words_result   – text lines with optional merged formulas
      - formula_result           – standalone formula regions
      - layouts                  – document layout regions (figure, table, etc.)
    """
    api_key = os.environ["BAIDU_API_KEY"]
    secret_key = os.environ["BAIDU_SECRET_KEY"]
    token = _get_access_token(api_key, secret_key)

    image_b64 = base64.b64encode(png_bytes).decode()

    resp = httpx.post(
        BAIDU_OCR_URL,
        params={"access_token": token},
        data={
            "image": image_b64,
            "language_type": "CHN_ENG",
            "recg_formula": "true",
            "layout_analysis": "true",
        },
        headers={"content-type": "application/x-www-form-urlencoded"},
        timeout=60,
    )
    resp.raise_for_status()
    result = resp.json()

    if "error_code" in result:
        raise RuntimeError(f"Baidu OCR error {result['error_code']}: {result.get('error_msg')}")

    return result


# ── Post-processing ───────────────────────────────────────────────────────────


def extract_text_and_figures(ocr_result: dict, page_png: bytes) -> tuple[str, list[bytes]]:
    """
    Post-process an OCR result into:

    combined_text : str
        Linear text of the page.  When ``words_result`` is present (i.e.
        ``recg_formula=true`` was used) the text already contains inline LaTeX
        formulas.  Falls back to ``results`` otherwise.

    figures : list[bytes]
        PNG crops of every ``figure`` region detected by layout analysis.
        Each crop is returned as raw PNG bytes so callers can embed or store them.

    Args:
        ocr_result: Raw JSON dict returned by :func:`ocr_image`.
        page_png:   Original PNG bytes of the page (used for cropping figures).
    """
    lines: list[str] = []

    # Prefer words_result (text + inline formulas merged by Baidu)
    if ocr_result.get("words_result"):
        for item in ocr_result["words_result"]:
            word = item.get("words", "")
            if word:
                lines.append(word)
    else:
        # Fallback: plain text lines
        for item in ocr_result.get("results", []):
            word = item.get("words", {}).get("word", "")
            if word:
                lines.append(word)

    combined_text = "\n".join(lines)

    # Crop figure regions
    figures: list[bytes] = []
    layouts = ocr_result.get("layouts", [])
    if layouts and page_png:
        try:
            from PIL import Image

            img = Image.open(io.BytesIO(page_png))
            for layout in layouts:
                if layout.get("layout") != "figure":
                    continue
                pts = layout.get("layout_location", [])
                if len(pts) < 4:
                    continue
                xs = [p["x"] for p in pts]
                ys = [p["y"] for p in pts]
                box = (min(xs), min(ys), max(xs), max(ys))
                cropped = img.crop(box)
                buf = io.BytesIO()
                cropped.save(buf, format="PNG")
                figures.append(buf.getvalue())
        except ImportError:
            # Pillow not installed; skip figure cropping
            pass

    return combined_text, figures
