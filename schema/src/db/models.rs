//! [`sqlx::FromRow`] structs for every table defined in the initial migration.
//!
//! Each struct mirrors the column set for its table exactly, using Rust types
//! that SQLx can decode directly from PostgreSQL without any custom codec.
//!
//! Naming convention: `Db<TablePascalCase>Row`.

use sqlx::FromRow;

use super::types::{
    DbElementKind, DbElementalProblemKind, DbFontSize, DbImageFormat, DbImageKind,
    DbOrderFormat, DbOrderType, DbQuestionBlockKind,
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
// ── problem_categories ─────────────────────────────────────────────────────────

/// Row in the `problem_categories` table.
#[derive(Debug, Clone, FromRow)]
pub struct DbProblemCategoryRow {
    pub id: i64,
    pub cirriculum: i32,
    pub subject: i32,
    pub grade: i32,
    /// Free-form taxonomy tags, e.g. `["Algebra", "Quadratics"]`.
    pub categories: Vec<String>,
    /// Optional FK into an external `problem_origin` table.
    pub origin: Option<i32>,
}

// ── elemental_questions ─────────────────────────────────────────────────────────

/// Row in the `elemental_questions` table.
///
/// Block columns populated per `block_kind`:
///
/// | `block_kind` | non-NULL columns   |
/// |--------------|--------------------|
/// | `essay`      | `block_lines`      |
/// | `proof`      | `block_space`      |
/// | `solve`      | `block_space`      |
/// | `none`       | (both NULL)        |
#[derive(Debug, Clone, FromRow)]
pub struct DbElementalQuestionRow {
    /// Application-assigned string identifier (e.g. `"q1a"`).
    pub id: String,
    /// FK → `paragraphs.id` — question body.
    pub content_id: i64,
    /// FK → `paragraphs.id` — optional model answer (hidden in Problem view).
    pub answer_id: Option<i64>,
    /// FK → `paragraphs.id` — optional worked solution.
    pub solution_id: Option<i64>,
    /// FK → `lists.id` — optional multiple-choice pool.
    pub choice_pool_id: Option<i64>,
    pub block_kind: DbQuestionBlockKind,
    /// `Essay`: number of ruled answer lines.
    pub block_lines: Option<i32>,
    /// `Proof` / `Solve`: vertical space in rem.
    pub block_space: Option<f32>,
}

// ── elemental_problems ─────────────────────────────────────────────────────────────

/// Row in the `elemental_problems` table.
///
/// `kind` is always `'question'`; `question_id` is always non-NULL.
#[derive(Debug, Clone, FromRow)]
pub struct DbElementalProblemRow {
    pub id: i64,
    pub kind: DbElementalProblemKind,
    /// FK → `elemental_questions.id`.
    pub question_id: Option<String>,
}

// ── single_problems ─────────────────────────────────────────────────────────────

/// Row in the `single_problems` table.
#[derive(Debug, Clone, FromRow)]
pub struct DbSingleProblemRow {
    pub id: i64,
    /// FK → `elemental_problems.id`.
    pub problem_id: i64,
    /// FK → `problem_categories.id`.
    pub category_id: i64,
}

// ── problem_groups ──────────────────────────────────────────────────────────────

/// Row in the `problem_groups` table.
#[derive(Debug, Clone, FromRow)]
pub struct DbProblemGroupRow {
    pub id: i64,
    /// FK → `paragraphs.id` — shared reading material for all sub-problems.
    pub material_id: i64,
    /// FK → `problem_categories.id`.
    pub category_id: i64,
}

/// Row in the `problem_group_items` join table.
///
/// `position` is 0-based and forms a gapless sequence per `group_id`.
#[derive(Debug, Clone, FromRow)]
pub struct DbProblemGroupItemRow {
    pub group_id: i64,
    pub position: i32,
    /// FK → `elemental_problems.id`.
    pub problem_id: i64,
}