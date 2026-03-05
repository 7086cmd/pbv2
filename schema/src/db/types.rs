//! PostgreSQL enum types that mirror the `CREATE TYPE` declarations in the
//! migration.  Each type derives [`sqlx::Type`] so it can be used directly in
//! query macros and [`sqlx::FromRow`] structs.
//!
//! `From` / `TryFrom` implementations are provided to cheaply convert between
//! these DB-layer enums and their counterparts in the domain model.

use sqlx::Type;

use crate::schema::elements::{FontSize, ImageFormat, OrderFormat, OrderType};

// ── font_size ─────────────────────────────────────────────────────────────────

/// Maps to the `font_size` PostgreSQL enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Type)]
#[sqlx(type_name = "font_size", rename_all = "snake_case")]
pub enum DbFontSize {
    Tiny,
    Script,
    Footnote,
    Small,
    Normal,
    Large,
    XLarge,
    XxLarge,
}

impl From<FontSize> for DbFontSize {
    fn from(v: FontSize) -> Self {
        match v {
            FontSize::Tiny => DbFontSize::Tiny,
            FontSize::Script => DbFontSize::Script,
            FontSize::Footnote => DbFontSize::Footnote,
            FontSize::Small => DbFontSize::Small,
            FontSize::Normal => DbFontSize::Normal,
            FontSize::Large => DbFontSize::Large,
            FontSize::XLarge => DbFontSize::XLarge,
            FontSize::XXLarge => DbFontSize::XxLarge,
        }
    }
}

impl From<DbFontSize> for FontSize {
    fn from(v: DbFontSize) -> Self {
        match v {
            DbFontSize::Tiny => FontSize::Tiny,
            DbFontSize::Script => FontSize::Script,
            DbFontSize::Footnote => FontSize::Footnote,
            DbFontSize::Small => FontSize::Small,
            DbFontSize::Normal => FontSize::Normal,
            DbFontSize::Large => FontSize::Large,
            DbFontSize::XLarge => FontSize::XLarge,
            DbFontSize::XxLarge => FontSize::XXLarge,
        }
    }
}

// ── image_format ──────────────────────────────────────────────────────────────

/// Maps to the `image_format` PostgreSQL enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Type)]
#[sqlx(type_name = "image_format", rename_all = "snake_case")]
pub enum DbImageFormat {
    Png,
    Jpeg,
}

impl From<ImageFormat> for DbImageFormat {
    fn from(v: ImageFormat) -> Self {
        match v {
            ImageFormat::Png => DbImageFormat::Png,
            ImageFormat::Jpeg => DbImageFormat::Jpeg,
        }
    }
}

impl From<DbImageFormat> for ImageFormat {
    fn from(v: DbImageFormat) -> Self {
        match v {
            DbImageFormat::Png => ImageFormat::Png,
            DbImageFormat::Jpeg => ImageFormat::Jpeg,
        }
    }
}

// ── image_kind ────────────────────────────────────────────────────────────────

/// Maps to the `image_kind` PostgreSQL enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Type)]
#[sqlx(type_name = "image_kind", rename_all = "snake_case")]
pub enum DbImageKind {
    Binary,
    Url,
    Latex,
    PdfSvg,
}

// ── order_type ────────────────────────────────────────────────────────────────

/// Maps to the `order_type` PostgreSQL enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Type)]
#[sqlx(type_name = "order_type", rename_all = "snake_case")]
pub enum DbOrderType {
    UppercaseAlphabetic,
    LowercaseAlphabetic,
    UppercaseRoman,
    LowercaseRoman,
    Decimal,
    Unordered,
}

impl From<OrderType> for DbOrderType {
    fn from(v: OrderType) -> Self {
        match v {
            OrderType::UppercaseAlphabetic => DbOrderType::UppercaseAlphabetic,
            OrderType::LowercaseAlphabetic => DbOrderType::LowercaseAlphabetic,
            OrderType::UppercaseRoman => DbOrderType::UppercaseRoman,
            OrderType::LowercaseRoman => DbOrderType::LowercaseRoman,
            OrderType::Decimal => DbOrderType::Decimal,
            OrderType::Unordered => DbOrderType::Unordered,
        }
    }
}

impl From<DbOrderType> for OrderType {
    fn from(v: DbOrderType) -> Self {
        match v {
            DbOrderType::UppercaseAlphabetic => OrderType::UppercaseAlphabetic,
            DbOrderType::LowercaseAlphabetic => OrderType::LowercaseAlphabetic,
            DbOrderType::UppercaseRoman => OrderType::UppercaseRoman,
            DbOrderType::LowercaseRoman => OrderType::LowercaseRoman,
            DbOrderType::Decimal => OrderType::Decimal,
            DbOrderType::Unordered => OrderType::Unordered,
        }
    }
}

// ── order_format ──────────────────────────────────────────────────────────────

/// Maps to the `order_format` PostgreSQL enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Type)]
#[sqlx(type_name = "order_format", rename_all = "snake_case")]
pub enum DbOrderFormat {
    Period,
    Parenthesis,
    RightParenthesis,
    None,
}

impl From<OrderFormat> for DbOrderFormat {
    fn from(v: OrderFormat) -> Self {
        match v {
            OrderFormat::Period => DbOrderFormat::Period,
            OrderFormat::Parenthesis => DbOrderFormat::Parenthesis,
            OrderFormat::RightParenthesis => DbOrderFormat::RightParenthesis,
            OrderFormat::None => DbOrderFormat::None,
        }
    }
}

impl From<DbOrderFormat> for OrderFormat {
    fn from(v: DbOrderFormat) -> Self {
        match v {
            DbOrderFormat::Period => OrderFormat::Period,
            DbOrderFormat::Parenthesis => OrderFormat::Parenthesis,
            DbOrderFormat::RightParenthesis => OrderFormat::RightParenthesis,
            DbOrderFormat::None => OrderFormat::None,
        }
    }
}

// ── element_kind ──────────────────────────────────────────────────────────────

/// Maps to the `element_kind` PostgreSQL enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Type)]
#[sqlx(type_name = "element_kind", rename_all = "snake_case")]
pub enum DbElementKind {
    Text,
    Table,
    Image,
    List,
    Blank,
}
