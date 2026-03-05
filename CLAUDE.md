# CLAUDE.md — PBV2 Codebase Guide

## Project Overview

**PBV2** is a problem/paper builder application targeting educators and assessment authors. It models rich document elements (text, tables, images, lists, blanks, multiple-choice pools) and renders them to LaTeX, HTML, or Markdown. The stack is:

- **Backend / core library**: Rust (`schema` crate) — domain model + multi-target renderer + PostgreSQL persistence layer
- **Frontend**: Vue 3 + TypeScript + Vite, packaged as a **Tauri** desktop application (`client/`)
- **Database**: PostgreSQL, managed via plain SQL migrations (`migrations/`)

---

## Repository Layout

```
pbv2/
├── Cargo.toml              # Workspace root — members: [client/src-tauri, schema]
├── Cargo.lock
├── migrations/
│   └── 0001_initial.sql    # Full PostgreSQL schema (run once against the DB)
├── schema/                 # Core Rust library crate
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          # Public re-exports for the entire crate
│       ├── db/             # SQLx row structs + PostgreSQL enum types
│       │   ├── mod.rs
│       │   ├── models.rs   # DbXxxRow structs (FromRow)
│       │   └── types.rs    # DbXxx enums (sqlx::Type) with From<domain> impls
│       ├── engines/        # LaTeX compilation engine trait + impls
│       │   ├── mod.rs      # Engine trait (async compile / compile_once)
│       │   ├── builtin.rs  # BuiltinEngine impl
│       │   └── xelatex.rs  # XeLaTeX impl
│       ├── latex/
│       │   ├── mod.rs
│       │   └── builder.rs  # LatexBuilder + DocumentClass — assembles full .tex docs
│       ├── preambles/
│       │   ├── preamble.tex    # Base LaTeX preamble (always included)
│       │   └── document.tex    # Extra preamble for non-Standalone classes
│       └── schema/
│           ├── mod.rs
│           ├── renderer.rs     # RenderTarget / RenderEnvironment traits + blanket impls
│           ├── utils/
│           │   └── svg.rs
│           └── elements/
│               ├── mod.rs      # Public re-exports for all element types
│               ├── text.rs     # Text, TextFormat, TextFlags, FontSize, TextAttributes
│               ├── image.rs    # Image enum (Binary/Url/Latex/PdfSvg), BinaryImage, ImageFormat
│               ├── compiled.rs # CompiledGraph — pre-rendered TikZ / PGF graphs
│               ├── table.rs    # Table, Cell
│               ├── list.rs     # List, OrderType, OrderFormat
│               ├── blank.rs    # Blank, BlankAnswer (fill-in-the-blank elements)
│               ├── choice.rs   # Choice, ChoicePool (multiple-choice option sets)
│               └── element.rs  # Element enum, Paragraph — composes all element types
└── client/                 # Tauri + Vue 3 desktop application
    ├── package.json
    ├── pnpm-workspace.yaml
    ├── vite.config.ts
    ├── vitest.config.ts
    ├── tsconfig.json / tsconfig.app.json / tsconfig.node.json / tsconfig.vitest.json
    ├── eslint.config.ts
    ├── src/
    │   ├── main.ts         # Vue app entry point
    │   ├── App.vue         # Root component
    │   ├── router/
    │   │   └── index.ts    # vue-router (currently no routes defined)
    │   ├── stores/
    │   │   └── counter.ts  # Example Pinia store
    │   └── __tests__/
    │       └── App.spec.ts # Vitest component tests
    └── src-tauri/          # Rust Tauri backend (shell)
        ├── Cargo.toml
        └── src/
            ├── main.rs     # Tauri entry point
            └── lib.rs      # Tauri builder setup + plugin config
```

---

## Core Domain Model (`schema` crate)

### Renderer Trait System

All rendering flows through a generic trait defined in `schema/src/schema/renderer.rs`:

```rust
pub trait Renderer<T: RenderTarget, E: RenderEnvironment> {
    fn render(&self) -> anyhow::Result<String>;
}
```

**RenderTargets** (output format):
- `Latex` — LaTeX source
- `Html` — HTML markup
- `Markdown` — CommonMark Markdown
- `Proprietary` — reserved

**RenderEnvironments** (document context):
- `Problem` — blank slots rendered as empty underlines/hspaces
- `Solution` — blank slots rendered with their answers
- `Universal` — delegates to `Problem` for most types (see below)

**Blanket rule**: Any type that implements `Renderer<T, Universal>` automatically gets `Renderer<T, Problem>` and `Renderer<T, Solution>` for free (both delegate to the `Universal` impl). The only exception is `Blank`, which only implements `Renderer<T, Problem>` and `Renderer<T, Solution>` directly — it has no `Universal` impl. When `Element::Blank` is rendered in a `Universal` context, the element dispatcher forwards to `Problem`.

### Element Types

All element types live in `schema/src/schema/elements/`:

| Type | Description |
|------|-------------|
| `Text` | Rich-text run with per-segment formatting (bold, italic, underline, superscript, subscript, monospace, formula, colour flags). Parsed from a compact inline markup syntax. |
| `Table` | Grid of `Cell`s, each cell containing a `Text`. Supports row/col spans and header cells. |
| `Image` | Enum: `Binary(BinaryImage)`, `Url(String)`, `Latex(CompiledGraph)`, `PdfSvg(PdfSvgImage)`. |
| `List` | Ordered or unordered list of `Paragraph`s. Configurable `OrderType` and `OrderFormat`. |
| `Blank` | Fill-in-the-blank slot with a mark value, width, and a `BlankAnswer`. |
| `ChoicePool` | Set of `Choice` items (Text or Image) with ordering — renders as a list. Converts to/from `List`. |
| `Element` | Top-level discriminated union: `Text | Table | Image | List | Blank`. |
| `Paragraph` | Sequence of `Element`s, optionally followed by a `ChoicePool`. |

### Text Formatting

`Text` uses bitflags (`TextFlags`) to encode per-run formatting. The inline markup syntax uses `\[flags]{content}` notation:

| Flag indicator | Meaning |
|---|---|
| `b` | Bold |
| `i` | Italic |
| `u` | Underline |
| `w` | Underwave (wavy underline) |
| `d` | Strikethrough ("delete") |
| `s` | Superscript |
| `x` | Subscript |
| `m` | Monospace |
| `f` | Formula (math mode) |
| `r` | Red colour |

The `formatting` field is a packed binary sequence of 8-byte `TextFormat` segments stored as `BYTEA` in PostgreSQL (little-endian u16 language, u32 start offset, u16 flags).

### Blank Answers

`BlankAnswer` supports three variants:
- `Text(Text)` — free-text answer
- `SingleChoice(usize, OrderType)` — index into a `ChoicePool`
- `MultipleChoice(Vec<usize>, OrderType)` — multiple indices

### LaTeX Builder

`LatexBuilder` assembles a complete `.tex` document:
- Always includes `preambles/preamble.tex`
- Adds `preambles/document.tex` for non-Standalone document classes
- Supports `Article { toc }`, `Report`, `Book`, `Standalone`, `Subfile`

### LaTeX Compilation Engines

The `Engine` trait (`engines/mod.rs`) provides `compile_once` and `compile` (with optional BibTeX). Implementations:
- `BuiltinEngine` — uses a built-in renderer
- `XeLaTeX` — shells out to `xelatex`

The `compile` method runs 2–3 passes automatically (more when BibTeX is needed).

---

## Database Schema (`migrations/0001_initial.sql`)

PostgreSQL with a discriminated-union approach. Key tables:

| Table | Purpose |
|-------|---------|
| `texts` | Text runs with packed binary `formatting` + UTF-8 `content` BYTEA |
| `images` | Single-table discriminator; variant columns are nullable |
| `blanks` | Application-assigned `id` (not BIGSERIAL); references `texts.answer_id` |
| `tables` | Dimensions only (`rows`, `cols`) |
| `table_cells` | Composite PK `(table_id, row, col)`; covered cells are absent |
| `lists` | `order_type` + `order_format` enums |
| `paragraphs` | Simple identity table (id only) |
| `list_items` | Ordered join: `list_id → paragraph_id` with 0-based `position` |
| `elements` | Top-level discriminated union; exactly one FK is non-NULL |
| `paragraph_elements` | Ordered join: `paragraph_id → element_id` with 0-based `position` |

**DB layer types** (`schema/src/db/`):
- Row structs follow the naming convention `Db<TablePascalCase>Row` and derive `sqlx::FromRow`
- Enum types follow `Db<TypePascalCase>` and derive `sqlx::Type`
- Bidirectional `From<Domain>` / `From<Db>` conversions are implemented in `types.rs`

---

## Frontend (`client/`)

Vue 3 + TypeScript SPA, bundled by Vite, distributed as a Tauri desktop app.

**Key dependencies:**
- `vue` ^3.5, `vue-router` ^5, `pinia` ^3 — UI + state
- `@tauri-apps/api` ^2 — Tauri JS bridge
- `vite` (beta), `@vitejs/plugin-vue` ^6 — build
- `vitest` ^4, `@vue/test-utils` ^2, `jsdom` — testing
- `oxlint` + `eslint` — linting
- `oxfmt` — formatting

**Path alias:** `@` resolves to `client/src/`.

**Package manager:** `pnpm` (see `pnpm-workspace.yaml`).

**Node requirement:** `^20.19.0 || >=22.12.0`

---

## Development Workflows

### Rust (`schema` crate)

```bash
# Run all tests
cargo test -p schema

# Check compilation
cargo check

# Build
cargo build

# Clippy lints
cargo clippy -- -D warnings

# Format
cargo fmt
```

Tests live alongside source files in `#[cfg(test)] mod tests` blocks. Each element module has comprehensive unit tests covering all three render targets × all environments.

### Frontend (`client/`)

All commands run from `client/`:

```bash
# Install dependencies
pnpm install

# Development server (hot reload)
pnpm dev

# Production build (type-check + bundle)
pnpm build

# Type-check only
pnpm type-check

# Unit tests (Vitest)
pnpm test:unit

# Lint (oxlint → eslint, both with --fix)
pnpm lint

# Format (oxfmt)
pnpm format
```

### Tauri Desktop App

```bash
# From client/ — dev mode with Tauri shell
pnpm tauri dev

# Production build
pnpm tauri build
```

### Database

Migrations are raw SQL files in `migrations/`. Apply them in order against a PostgreSQL instance. There is no ORM migration runner configured yet — apply `0001_initial.sql` manually:

```bash
psql -d <database> -f migrations/0001_initial.sql
```

---

## Coding Conventions

### Rust

- **Edition**: 2024 (workspace), 2021 (Tauri crate)
- **Error handling**: `anyhow::Result<String>` throughout the renderer layer
- **Async**: `tokio` full runtime; engine compilation is `async` via `async-trait`
- **Renderer impls**: Each element implements `Renderer<T, E>` separately for each target/environment combination. Do not add `Universal` to types that require distinct Problem vs Solution behaviour (like `Blank`).
- **DB naming**: Row structs → `Db<Name>Row`; enum types → `Db<Name>`
- **Position fields**: Always 0-based; must form a gapless sequence per parent
- **Discriminated unions in DB**: Exactly one FK column is non-NULL per row; enforced with `CHECK` constraints

### TypeScript / Vue

- **Composition API** (`<script setup>`) is the standard for Vue components
- **Pinia** for state management — define stores in `src/stores/`
- **vue-router** for navigation — register routes in `src/router/index.ts`
- **`@` alias** for imports from `src/`
- **Linting order**: oxlint runs before eslint (both auto-fix); format with oxfmt before committing

### Testing

- **Rust**: Unit tests co-located with source (`#[cfg(test)]`); test every render target and environment variant
- **Vue**: Vitest + `@vue/test-utils` in `src/__tests__/`; use `jsdom` environment

---

## Key Architectural Decisions

1. **Renderer blanket impls**: `Problem` and `Solution` blanket-delegate to `Universal` — add `Universal` impls, not `Problem`/`Solution`, unless the element needs genuinely different output per environment.

2. **`Blank` is environment-sensitive**: It has no `Universal` impl. `Element` dispatches `Blank` variants explicitly to `Problem` in `Universal` context. If you add a new environment-sensitive element, follow the same pattern.

3. **Single-table discriminator for `images`**: Variant columns are nullable; DB constraints enforce exactly-one-variant population. When adding a new image variant, add columns + a new `CHECK` constraint in a migration.

4. **Text encoding**: `formatting` is a packed binary blob, not JSON or separate rows, for performance. Inspect `TextFormat` in `text.rs` for the 8-byte layout.

5. **`ChoicePool` ↔ `List`**: A `ChoicePool` is a view over a `List` (same ordering semantics). `TryFrom<List>` validates that every item has exactly one Text or Image element. Rendering delegates to `List`.

6. **Workspace structure**: `schema` is a pure library; `client/src-tauri` is the thin Tauri shell. Business logic belongs in `schema`.
