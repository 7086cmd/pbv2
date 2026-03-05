//! [`sqlx::FromRow`] structs for every table defined in the initial migration.
//!
//! Each struct mirrors the column set for its table exactly, using Rust types
//! that SQLx can decode directly from PostgreSQL without any custom codec.
//!
//! Naming convention: `Db<TablePascalCase>Row`.

use sqlx::FromRow;

use super::types::{
    DbElementKind, DbFontSize, DbImageFormat, DbImageKind, DbOrderFormat, DbOrderType,
};

// ── texts ─────────────────────────────────────────────────────────────────────

/// Row in the `texts` table.
///
/// `formatting` is a packed array of 8-byte [`crate::TextFormat`] segments
/// (little-endian). `content` is the raw UTF-8 payload of all text runs
/// concatenated; run boundaries are derived from the `start` pointers inside
/// `formatting`.
#[derive(Debug, Clone, FromRow)]
pub struct DbTextRow {
    pub id: i64,
    pub formatting: Vec<u8>,
    pub content: Vec<u8>,
    pub font_size: DbFontSize,
    /// Red channel (0-255) stored as `SMALLINT`.
    pub color_r: i16,
    /// Green channel (0-255) stored as `SMALLINT`.
    pub color_g: i16,
    /// Blue channel (0-255) stored as `SMALLINT`.
    pub color_b: i16,
}

// ── images ────────────────────────────────────────────────────────────────────

/// Row in the `images` table.
///
/// Only the columns relevant to the row's [`DbImageKind`] discriminator are
/// populated; all other variant-specific columns are `NULL`.
///
/// | `kind`    | non-NULL columns                                         |
/// |-----------|----------------------------------------------------------|
/// | `binary`  | `buffer`, `format`; optionally `filename`, `width_ratio` |
/// | `url`     | `url`                                                     |
/// | `latex`   | `tex_code`; optionally `svg_content`, `png_content`      |
/// | `pdf_svg` | `pdf_buffer`, `svg_content`; optionally `width_ratio`    |
#[derive(Debug, Clone, FromRow)]
pub struct DbImageRow {
    pub id: i64,
    pub kind: DbImageKind,

    // Binary
    pub buffer: Option<Vec<u8>>,
    pub format: Option<DbImageFormat>,
    pub filename: Option<String>,

    // Url
    pub url: Option<String>,

    // Latex (CompiledGraph)
    pub tex_code: Option<String>,
    pub png_content: Option<Vec<u8>>,

    // Shared by Latex + PdfSvg
    pub svg_content: Option<String>,

    // Shared by Binary + PdfSvg
    pub width_ratio: Option<f64>,

    // PdfSvg
    pub pdf_buffer: Option<Vec<u8>>,
}

// ── blanks ────────────────────────────────────────────────────────────────────

/// Row in the `blanks` table.
///
/// `id` is application-assigned (see `Blank::id: i32`).
#[derive(Debug, Clone, FromRow)]
pub struct DbBlankRow {
    pub id: i32,
    pub mark: f32,
    /// FK → `texts.id`; holds the correct answer.
    pub answer_id: i64,
    pub width: f32,
}

// ── tables ────────────────────────────────────────────────────────────────────

/// Row in the `tables` table.
#[derive(Debug, Clone, FromRow)]
pub struct DbTableRow {
    pub id: i64,
    pub rows: i32,
    pub cols: i32,
}

// ── table_cells ───────────────────────────────────────────────────────────────

/// Row in the `table_cells` table.
///
/// The composite primary key is `(table_id, row, col)`.
#[derive(Debug, Clone, FromRow)]
pub struct DbCellRow {
    pub table_id: i64,
    pub row: i32,
    pub col: i32,
    /// FK → `texts.id`; the rendered content of this cell.
    pub content_id: i64,
    pub row_span: i32,
    pub col_span: i32,
    pub header: bool,
}

// ── lists ─────────────────────────────────────────────────────────────────────

/// Row in the `lists` table.
#[derive(Debug, Clone, FromRow)]
pub struct DbListRow {
    pub id: i64,
    pub order_type: DbOrderType,
    pub order_format: DbOrderFormat,
}

// ── paragraphs ────────────────────────────────────────────────────────────────

/// Row in the `paragraphs` table.
#[derive(Debug, Clone, FromRow)]
pub struct DbParagraphRow {
    pub id: i64,
}

// ── list_items ────────────────────────────────────────────────────────────────

/// Row in the `list_items` join table.
///
/// `position` is 0-based and forms a gapless sequence per `list_id`.
#[derive(Debug, Clone, FromRow)]
pub struct DbListItemRow {
    pub list_id: i64,
    pub position: i32,
    /// FK → `paragraphs.id`.
    pub paragraph_id: i64,
}

// ── elements ──────────────────────────────────────────────────────────────────

/// Row in the `elements` table.
///
/// Exactly one of the FK columns is non-NULL, matching the `kind`
/// discriminator.
#[derive(Debug, Clone, FromRow)]
pub struct DbElementRow {
    pub id: i64,
    pub kind: DbElementKind,
    pub text_id: Option<i64>,
    pub table_id: Option<i64>,
    pub image_id: Option<i64>,
    pub list_id: Option<i64>,
    pub blank_id: Option<i32>,
}

// ── paragraph_elements ────────────────────────────────────────────────────────

/// Row in the `paragraph_elements` join table.
///
/// `position` is 0-based and forms a gapless sequence per `paragraph_id`.
#[derive(Debug, Clone, FromRow)]
pub struct DbParagraphElementRow {
    pub paragraph_id: i64,
    pub position: i32,
    /// FK → `elements.id`.
    pub element_id: i64,
}
