# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

PBv2 is a problem bank application for exam papers. It has three main components:

1. **`schema/`** – Rust crate defining the core data model and renderers
2. **`client/`** – Tauri desktop app (Vue 3 frontend + Rust backend)
3. **`importer/`** – Python CLI to ingest exam papers (PDF/DOCX/TeX/Markdown) into a PostgreSQL database

## Commands

### Client (Tauri + Vue)

Run from `client/` using `pnpm`:

```bash
pnpm dev              # start Vite dev server (frontend only)
pnpm tauri dev        # start full Tauri app with hot-reload
pnpm build            # type-check + Vite build
pnpm tauri build      # build native Tauri app
pnpm test:unit        # run Vitest unit tests
pnpm lint             # oxlint + eslint (with auto-fix)
pnpm format           # oxfmt formatter
```

Run a single test file:

```bash
pnpm test:unit src/__tests__/foo.spec.ts
```

### Schema (Rust crate)

From the workspace root:

```bash
cargo build -p schema
cargo test -p schema
cargo test -p schema -- test_name   # single test
```

### Importer (Python)

From `importer/` using `uv`:

```bash
uv run python main.py import-pdf path/to/paper.pdf --category-id 1
uv run python main.py import-doc path/to/paper.docx --category-id 1
uv run python main.py import-pdf path/to/paper.pdf --dry-run   # skip DB, print JSON
```

## Architecture

### Schema crate (`schema/`)

The central Rust library shared by the Tauri backend. Key abstractions:

- **`Element` / `Paragraph`** – building blocks for rich content (text runs with formatting, images, tables, lists, LaTeX blanks, SI units, chemistry, code listings).
- **`ElementalQuestion`** – an atomic question with optional answer, solution, choice pool, and an answer-block type (`Essay`, `Solve`, `Proof`, `None`).
- **`ElementalProblem`** – either a single `Question` or a `QuestionSeries` (shared intro + sub-questions).
- **`SingleProblem`** / **`ProblemGroup`** – top-level problem wrappers that carry a `ProblemCategory` (curriculum, subject, grade, topic tags).
- **`Renderer<T: RenderTarget, E: RenderEnvironment>`** – generic render trait. Targets: `Html`, `Latex`, `Markdown`. Environments: `Problem`, `Solution`, `Universal`. Implementing `Renderer<T, Universal>` automatically gives `Problem` and `Solution` impls.
- **`db/`** – `sqlx`-based row structs (`Db*Row`) for every PostgreSQL table, used for reading back stored data.

### Tauri backend (`client/src-tauri/`)

Thin glue layer. `commands.rs` exposes `#[tauri::command]` functions that construct schema types and call `Renderer::render()`, returning HTML strings to the frontend. All commands are registered in `lib.rs`.

### Vue frontend (`client/src/`)

- `shared/api.ts` – typed wrappers around `invoke()` calls to Tauri commands.
- `stores/` – Pinia stores.
- `router/` – Vue Router (currently empty routes, to be built out).

### Importer (`importer/`)

Two pipelines sharing a common `_run_chunks()` loop:

**PDF pipeline** (`run_pipeline`): PDF → PNG pages (PyMuPDF) → OCR per page (Baidu API) → group into chunks of N pages → LLM extraction → DB store.

**Document pipeline** (`run_doc_pipeline`): DOCX/TeX/Markdown → plain text + LaTeX math (pypandoc) → chunk by character count → LLM extraction → DB store.

LLM layer (`llm.py`): `LLMBackend` ABC with `OpenAIBackend` (structured outputs) and `GeminiBackend` (JSON schema constrained). Selected via `LLM_PROVIDER` env var.

The LLM returns `ExtractedPage` (Pydantic, defined in `schema.py`), which `store.py` maps to the PostgreSQL schema.

### Database

PostgreSQL. Migrations are in `migrations/` (plain SQL, numbered `0001`–`0004`). The schema stores content as normalized rows: `texts`, `images`, `paragraphs`, `elements` (discriminated union via `kind` column), `elemental_questions`, `question_series`, `elemental_problems`, `single_problems`, `problem_groups`, `problem_categories`.

### Importer environment variables

Required in `importer/.env`:

```
DATABASE_URL=postgresql://user:pass@localhost/pbv2
BAIDU_API_KEY=...       # PDF OCR only
BAIDU_SECRET_KEY=...    # PDF OCR only
LLM_PROVIDER=openai     # or gemini
CHATGPT_API_KEY=...     # when LLM_PROVIDER=openai
GEMINI_API_KEY=...      # when LLM_PROVIDER=gemini
```
