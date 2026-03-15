# PBv2 justfile — run `just` to list available recipes

set dotenv-load := true

# List all recipes
default:
    @just --list

# ── Client (Tauri + Vue) ─────────────────────────────────────────────────────

# Start Vite dev server (frontend only)
dev:
    cd client && pnpm dev

# Start full Tauri app with hot-reload
tauri:
    cd client && pnpm tauri dev

# Type-check + Vite build
build:
    cd client && pnpm build

# Build native Tauri app
tauri-build:
    cd client && pnpm tauri build

# Run Vitest unit tests
test:
    cd client && pnpm test:unit

# Run a single test file
test-file file:
    cd client && pnpm test:unit {{ file }}

# Lint (oxlint + eslint with auto-fix)
lint:
    cd client && pnpm lint

# Format with oxfmt
fmt:
    cd client && pnpm format

# ── Schema (Rust crate) ──────────────────────────────────────────────────────

# Build schema crate
schema-build:
    cargo build -p schema

# Test schema crate
schema-test:
    cargo test -p schema

# Run a single schema test
schema-test-one name:
    cargo test -p schema -- {{ name }}

# ── Importer (Python) ────────────────────────────────────────────────────────

# Import a PDF paper
import-pdf path category_id:
    cd importer && uv run python main.py import-pdf {{ path }} --category-id {{ category_id }}

# Import a DOCX/TeX/Markdown paper
import-doc path category_id:
    cd importer && uv run python main.py import-doc {{ path }} --category-id {{ category_id }}

# Dry-run import (skip DB, print JSON)
import-dry path:
    cd importer && uv run python main.py import-pdf {{ path }} --dry-run

# ── Database ─────────────────────────────────────────────────────────────────

# Apply all pending migrations
migrate:
    @for f in migrations/*.sql; do \
        echo "Applying $f…"; \
        psql "$DATABASE_URL" -f "$f"; \
    done

# ── Workspace ────────────────────────────────────────────────────────────────

# Install all dependencies (client + root)
install:
    cd client && pnpm install

# Check everything: client lint/types + schema tests
check: lint schema-test
    @echo "All checks passed."
