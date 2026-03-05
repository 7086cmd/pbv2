//! # Text
//!
//! The `text` module defines the `Text` struct, which represents a piece of text in a PBV2 document. It contains the actual text content and any associated formatting information, such as font size, color, and style. The `Text` struct is used as a building block for creating more complex elements in the document, such as paragraphs and sections.
//!
//! The `text` is a hypertext element, and can be exported into various formats, such as LaTeX, HTML, Markdown, and future formats like Word.
//!
//! We allow the following formatting options for text, with bitflags to indicate which options are applied:
//! - **Bold**: Indicates that the text should be displayed in boldface. Indicator: `b`.
//! - **Italic**: Indicates that the text should be displayed in italics. Indicator: `i`.
//! - **Underline**: Indicates that the text should be underlined. Indicator: `u`.
//! - **Underwave**: Indicates that the text should have a wavy underline, often used to indicate spelling errors. Indicator: `w`, denoting "wave."
//! - **Strikethrough**: Indicates that the text should have a line through it. Indicator: `d`, denoting "delete."
//! - **Superscript**: Indicates that the text should be displayed as superscript. Indicator: `s`, denoting "superscript."
//! - **Subscript**: Indicates that the text should be displayed as subscript. Indicator: `x`, denoting "subscript."
//! - **Monospace**: Indicates that the text should be displayed in a monospace font. Indicator: `m`, denoting "monospace."
//! - **Formula**: Indicates that the text should be treated as a mathematical formula. Indicator: `f`, denoting "formula." In LaTeX, this would be rendered in math mode, while in HTML it would be rendered using MathJax or a similar library.
//! - **Red**: Indicates that the text should be displayed in red. Indicator: `r`, denoting "red."
//! For further formatting options, we can add more indicators as needed. The formatting options can be combined, so a piece of text could be both bold and italic, for example, which would be indicated by `bi`.
//!
//! The `Text` struct contains two vectors of `u8`. The first vector, `formatting`, contains the indicators for the formatting options that are applied to the text. Its structure is as follows:
//! - `BITS[0:2]``: Language of the text. The bit 0 indicates the primary language, and bit 1 indicates the secondary language.
//! - `BITS[2:6]`: The start pointer of the text in the document's text buffer. This indicates where the text begins in the overall document.
//! That is to say, the end of the previous text element is the start of the next text element, so we can use the start pointer to determine the length of the text by subtracting it from the start pointer of the next text element.
//! - `BITS[6:8]`: The formatting options applied to the text, represented as bitflags. Each bit corresponds to a specific formatting option, as described above. For example, if the text is bold and italic, the formatting byte would have bits 0 and 1 set (i.e., `0b00000011`).
//!
//! The second vector, `content`, contains the actual text content as a sequence of bytes. The text is encoded in UTF-8, so each character may be represented by one or more bytes. The length of the text can be determined by looking at the start pointer of the next text element, as mentioned above.
//!
//! We also provide some attributes for the whole `Text`, e.g., the font size (relative to the default font size), color, and other attributes that can be applied to the entire text element. These attributes are stored in a separate struct called `TextAttributes`, which is associated with the `Text` struct.
//!

#[derive(Debug, Clone)]
pub struct Text {
    /// A vector of bytes representing the formatting options and the start pointer of the text in the document's text buffer.
    pub formatting: Vec<u8>,
    /// A vector of bytes representing the actual text content, encoded in UTF-8.
    pub content: Vec<u8>,
    /// The attributes for the entire text element, such as font size and color.
    pub attributes: TextAttributes,
}

#[derive(Debug, Clone, Default)]
pub struct TextAttributes {
    /// The font size of the text, relative to the default font size. For example, a value of 1.0 means the default font size, while a value of 1.5 means 150% of the default font size.
    pub font_size: FontSize,
    /// The color of the text, represented as an RGB tuple (red, green, blue), where each component is a value between 0 and 255.
    pub color: (u8, u8, u8),
    // Other attributes can be added here as needed.
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum FontSize {
    /// `\\tiny` in LaTeX, which is typically 0.5 times the normal font size.
    Tiny,
    /// `\\scriptsize` in LaTeX, which is typically 0.7 times the normal font size.
    Script,
    /// `\\footnotesize` in LaTeX, which is typically 0.8 times the normal font size.
    Footnote,
    /// `\\small` in LaTeX, which is typically 0.9 times the normal font size.
    Small,
    /// `\\normalsize` in LaTeX, which is the default font size.
    #[default]
    Normal,
    /// `\\large` in LaTeX, which is typically 1.2 times the normal font size.
    Large,
    /// `\\Large` in LaTeX, which is typically 1.44 times the normal font size.
    XLarge,
    /// `\\LARGE` in LaTeX, which is typically 1.728 times the normal font size.
    XXLarge,
}

impl FontSize {
    pub fn ratio(&self) -> f32 {
        match self {
            FontSize::Tiny => 0.5,
            FontSize::Script => 0.7,
            FontSize::Footnote => 0.8,
            FontSize::Small => 0.9,
            FontSize::Normal => 1.0,
            FontSize::Large => 1.2,
            FontSize::XLarge => 1.44,
            FontSize::XXLarge => 1.728,
        }
    }
}

// ── TextFlags ─────────────────────────────────────────────────────────────────

bitflags::bitflags! {
    /// Formatting bitflags for a single text run, stored in `BITS[6:8]` of a
    /// [`TextFormat`] segment.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct TextFlags: u16 {
        /// `b` – Bold.
        const BOLD          = 1 << 0;
        /// `i` – Italic.
        const ITALIC        = 1 << 1;
        /// `u` – Underline.
        const UNDERLINE     = 1 << 2;
        /// `w` – Underwave (wavy underline).
        const UNDERWAVE     = 1 << 3;
        /// `d` – Strikethrough (delete).
        const STRIKETHROUGH = 1 << 4;
        /// `s` – Superscript.
        const SUPERSCRIPT   = 1 << 5;
        /// `x` – Subscript.
        const SUBSCRIPT     = 1 << 6;
        /// `m` – Monospace.
        const MONOSPACE     = 1 << 7;
        /// `f` – Formula (math mode).
        const FORMULA       = 1 << 8;
        /// `r` – Red colour.
        const RED           = 1 << 9;
    }
}

// ── TextFormat ────────────────────────────────────────────────────────────────

/// An eight-byte segment that encodes the language, start-pointer, and
/// formatting [`TextFlags`] for one run of text within a [`Text`] element.
///
/// Memory layout (little-endian):
/// ```text
/// bytes 0..=1  language  (u16)  – BITS[0:2]
/// bytes 2..=5  start     (u32)  – BITS[2:6], byte offset into Text::content
/// bytes 6..=7  flags     (u16)  – BITS[6:8], TextFlags bitflags
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextFormat {
    /// Language identifier; 0 = primary, 1 = secondary.
    pub language: u16,
    /// Byte offset of this run's text inside [`Text::content`].
    pub start: u32,
    /// Formatting options for this run.
    pub flags: TextFlags,
}

impl TextFormat {
    /// Deserialise one `TextFormat` from an eight-byte slice.
    ///
    /// # Panics
    /// Panics if `bytes.len() < 8`.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let language = u16::from_le_bytes([bytes[0], bytes[1]]);
        let start = u32::from_le_bytes([bytes[2], bytes[3], bytes[4], bytes[5]]);
        let raw = u16::from_le_bytes([bytes[6], bytes[7]]);
        TextFormat {
            language,
            start,
            flags: TextFlags::from_bits_truncate(raw),
        }
    }

    /// Serialise this `TextFormat` into an eight-byte array.
    pub fn to_bytes(self) -> [u8; 8] {
        let lang = self.language.to_le_bytes();
        let start = self.start.to_le_bytes();
        let flags = self.flags.bits().to_le_bytes();
        [
            lang[0], lang[1], start[0], start[1], start[2], start[3], flags[0], flags[1],
        ]
    }
}

// ── Text helpers ──────────────────────────────────────────────────────────────

impl Text {
    /// Parse a hypertext markup string into a [`Text`] value.
    ///
    /// # Syntax
    ///
    /// The format is inspired by LaTeX commands:
    ///
    /// ```text
    /// \[<flags>]{<content>}
    /// ```
    ///
    /// where `<flags>` is a sequence of one-letter indicators (combinable, e.g.
    /// `bi` for bold-italic) and `<content>` is the raw text for that run.
    /// Curly braces inside `<content>` may be nested.
    ///
    /// Plain text between formatted runs is emitted as a run with no flags.
    ///
    /// A lone `\` not followed by `[` is passed through as a literal backslash.
    ///
    /// # Flag letters
    ///
    /// | Letter | Flag           |
    /// |--------|----------------|
    /// | `b`    | BOLD           |
    /// | `i`    | ITALIC         |
    /// | `u`    | UNDERLINE      |
    /// | `w`    | UNDERWAVE      |
    /// | `d`    | STRIKETHROUGH  |
    /// | `s`    | SUPERSCRIPT    |
    /// | `x`    | SUBSCRIPT      |
    /// | `m`    | MONOSPACE      |
    /// | `f`    | FORMULA        |
    /// | `r`    | RED            |
    ///
    /// # Example
    ///
    /// ```text
    /// Hello \[b]{World} and \[bi]{this}.
    /// ```
    ///
    /// Produces three runs:
    /// 1. `"Hello "` – no flags
    /// 2. `"World"` – BOLD
    /// 3. `" and "` – no flags
    /// 4. `"this"` – BOLD | ITALIC
    /// 5. `"."` – no flags
    pub fn parse(input: &str) -> anyhow::Result<Self> {
        let mut content: Vec<u8> = Vec::new();
        let mut formatting: Vec<u8> = Vec::new();

        /// Append one run to the two output buffers.
        fn emit(content: &mut Vec<u8>, formatting: &mut Vec<u8>, text: &str, flags: TextFlags) {
            let start = content.len() as u32;
            let fmt = TextFormat {
                language: 0,
                start,
                flags,
            };
            formatting.extend_from_slice(&fmt.to_bytes());
            content.extend_from_slice(text.as_bytes());
        }

        let mut remaining = input;

        while !remaining.is_empty() {
            // ── Formatted run: \[flags]{content} ─────────────────────────────
            if remaining.starts_with("\\[") {
                remaining = &remaining[2..]; // skip `\[`

                // Parse flag letters up to `]`.
                let close_bracket = remaining
                    .find(']')
                    .ok_or_else(|| anyhow::anyhow!("missing ']' after format flags"))?;
                let flag_str = &remaining[..close_bracket];
                remaining = &remaining[close_bracket + 1..]; // skip past `]`

                let mut flags = TextFlags::empty();
                for ch in flag_str.chars() {
                    match ch {
                        'b' => flags |= TextFlags::BOLD,
                        'i' => flags |= TextFlags::ITALIC,
                        'u' => flags |= TextFlags::UNDERLINE,
                        'w' => flags |= TextFlags::UNDERWAVE,
                        'd' => flags |= TextFlags::STRIKETHROUGH,
                        's' => flags |= TextFlags::SUPERSCRIPT,
                        'x' => flags |= TextFlags::SUBSCRIPT,
                        'm' => flags |= TextFlags::MONOSPACE,
                        'f' => flags |= TextFlags::FORMULA,
                        'r' => flags |= TextFlags::RED,
                        other => anyhow::bail!("unknown format flag: {:?}", other),
                    }
                }

                // Expect `{`.
                if !remaining.starts_with('{') {
                    anyhow::bail!(
                        "expected '{{' after format flags, found {:?}",
                        remaining.chars().next()
                    );
                }
                remaining = &remaining[1..]; // skip `{`

                // Collect content up to the matching `}`, tracking nesting.
                let mut depth: usize = 1;
                let mut end: Option<usize> = None;
                for (i, c) in remaining.char_indices() {
                    match c {
                        '{' => depth += 1,
                        '}' => {
                            depth -= 1;
                            if depth == 0 {
                                end = Some(i);
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                let end =
                    end.ok_or_else(|| anyhow::anyhow!("unexpected end of input inside '{{...}}'"))?;
                let run_text = &remaining[..end];
                // Advance past the run content and the closing `}`.
                remaining = &remaining[end + 1..];

                emit(&mut content, &mut formatting, run_text, flags);

            // ── Literal backslash (not followed by `[`) ───────────────────────
            } else if remaining.starts_with('\\') {
                // Find the next `\[` or end-of-input; everything up to there
                // (including this backslash) is plain text.
                let plain_end = remaining[1..]
                    .find("\\[")
                    .map(|i| i + 1)
                    .unwrap_or(remaining.len());
                let plain = &remaining[..plain_end];
                remaining = &remaining[plain_end..];

                if !plain.is_empty() {
                    emit(&mut content, &mut formatting, plain, TextFlags::empty());
                }

            // ── Plain-text run ────────────────────────────────────────────────
            } else {
                let plain_end = remaining.find("\\[").unwrap_or(remaining.len());
                let plain = &remaining[..plain_end];
                remaining = &remaining[plain_end..];

                if !plain.is_empty() {
                    emit(&mut content, &mut formatting, plain, TextFlags::empty());
                }
            }
        }

        Ok(Text {
            formatting,
            content,
            attributes: TextAttributes::default(),
        })
    }

    /// Parse `self.formatting` into a `Vec<TextFormat>`.
    /// Each eight-byte chunk becomes one `TextFormat`.
    pub fn parse_formats(&self) -> Vec<TextFormat> {
        self.formatting
            .chunks(8)
            .filter(|c| c.len() == 8)
            .map(TextFormat::from_bytes)
            .collect()
    }

    /// Return the UTF-8 text slice for run `i` given the full list of formats.
    fn run_text<'a>(&'a self, formats: &[TextFormat], i: usize) -> anyhow::Result<&'a str> {
        let start = formats[i].start as usize;
        let end = if i + 1 < formats.len() {
            formats[i + 1].start as usize
        } else {
            self.content.len()
        };
        let end = end.min(self.content.len());
        Ok(std::str::from_utf8(
            self.content.get(start..end).unwrap_or(&[]),
        )?)
    }
}

impl std::str::FromStr for Text {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> anyhow::Result<Self> {
        Text::parse(s)
    }
}

// ── Renderer impls ────────────────────────────────────────────────────────────

use crate::schema::renderer::Universal;

use super::super::renderer::{Html, Latex, Markdown, Proprietary, Renderer};

impl Renderer<Latex, Universal> for Text {
    /// Render the `Text` element into a LaTeX fragment and write it to stdout.
    ///
    /// Global [`TextAttributes`]:
    /// - `font_size` → `{\tiny …}`, `{\large …}`, etc.  (`Normal` → no wrapper)
    /// - `color`     → `\textcolor[RGB]{r,g,b}{…}`       (`(0,0,0)` → no wrapper)
    ///
    /// Per-run [`TextFlags`]:
    /// | Flag           | LaTeX output                         |
    /// |----------------|--------------------------------------|
    /// | `FORMULA`      | `$…$`  (other flags ignored)         |
    /// | `BOLD`         | `\textbf{…}`                         |
    /// | `ITALIC`       | `\textit{…}`                         |
    /// | `UNDERLINE`    | `\underline{…}`                      |
    /// | `UNDERWAVE`    | `\uwave{…}`  *(requires ulem)*       |
    /// | `STRIKETHROUGH`| `\sout{…}`   *(requires ulem)*       |
    /// | `SUPERSCRIPT`  | `\textsuperscript{…}`                |
    /// | `SUBSCRIPT`    | `\textsubscript{…}`                  |
    /// | `MONOSPACE`    | `\texttt{…}`                         |
    /// | `RED`          | `\textcolor{red}{…}`                 |
    fn render(&self) -> anyhow::Result<String> {
        let formats = self.parse_formats();

        let size_cmd = match self.attributes.font_size {
            FontSize::Tiny => Some("\\tiny"),
            FontSize::Script => Some("\\scriptsize"),
            FontSize::Footnote => Some("\\footnotesize"),
            FontSize::Small => Some("\\small"),
            FontSize::Normal => None,
            FontSize::Large => Some("\\large"),
            FontSize::XLarge => Some("\\Large"),
            FontSize::XXLarge => Some("\\LARGE"),
        };

        let (cr, cg, cb) = self.attributes.color;
        let has_color = (cr, cg, cb) != (0, 0, 0);

        let mut out = String::new();

        if let Some(cmd) = size_cmd {
            out.push_str(&format!("{{{} ", cmd));
        }
        if has_color {
            out.push_str(&format!("\\textcolor[RGB]{{{},{},{}}}{{", cr, cg, cb));
        }

        for i in 0..formats.len() {
            let fmt = formats[i];
            let text = self.run_text(&formats, i)?;

            if fmt.flags.contains(TextFlags::FORMULA) {
                out.push('$');
                out.push_str(text);
                out.push('$');
            } else {
                let mut seg = text.to_owned();
                if fmt.flags.contains(TextFlags::RED) {
                    seg = format!("\\textcolor{{red}}{{{}}}", seg);
                }
                if fmt.flags.contains(TextFlags::MONOSPACE) {
                    seg = format!("\\texttt{{{}}}", seg);
                }
                if fmt.flags.contains(TextFlags::SUBSCRIPT) {
                    seg = format!("\\textsubscript{{{}}}", seg);
                }
                if fmt.flags.contains(TextFlags::SUPERSCRIPT) {
                    seg = format!("\\textsuperscript{{{}}}", seg);
                }
                if fmt.flags.contains(TextFlags::STRIKETHROUGH) {
                    seg = format!("\\sout{{{}}}", seg);
                }
                if fmt.flags.contains(TextFlags::UNDERWAVE) {
                    seg = format!("\\uwave{{{}}}", seg);
                }
                if fmt.flags.contains(TextFlags::UNDERLINE) {
                    seg = format!("\\underline{{{}}}", seg);
                }
                if fmt.flags.contains(TextFlags::ITALIC) {
                    seg = format!("\\textit{{{}}}", seg);
                }
                if fmt.flags.contains(TextFlags::BOLD) {
                    seg = format!("\\textbf{{{}}}", seg);
                }
                out.push_str(&seg);
            }
        }

        if has_color {
            out.push('}');
        }
        if size_cmd.is_some() {
            out.push('}');
        }

        Ok(out)
    }
}

impl Renderer<Html, Universal> for Text {
    /// Render the `Text` element into an HTML fragment and write it to stdout.
    ///
    /// Global [`TextAttributes`]:
    /// - `font_size` → `<span style="font-size: X%;">…</span>`
    /// - `color`     → `<span style="color:rgb(r,g,b);">…</span>`
    ///
    /// Per-run [`TextFlags`]:
    /// | Flag           | HTML output                                              |
    /// |----------------|----------------------------------------------------------|
    /// | `FORMULA`      | `$…$`  (MathJax handles it directly)                    |
    /// | `BOLD`         | `<strong>…</strong>`                                     |
    /// | `ITALIC`       | `<em>…</em>`                                             |
    /// | `UNDERLINE`    | `<span style="text-decoration:underline;">…</span>`      |
    /// | `UNDERWAVE`    | `<span style="text-decoration:underline wavy;">…</span>` |
    /// | `STRIKETHROUGH`| `<del>…</del>`                                           |
    /// | `SUPERSCRIPT`  | `<sup>…</sup>`                                           |
    /// | `SUBSCRIPT`    | `<sub>…</sub>`                                           |
    /// | `MONOSPACE`    | `<code>…</code>`                                         |
    /// | `RED`          | `<span style="color:red;">…</span>`                      |
    fn render(&self) -> anyhow::Result<String> {
        let formats = self.parse_formats();

        let size_pct = self.attributes.font_size.ratio() * 100.0;
        let has_size = self.attributes.font_size != FontSize::Normal;

        let (cr, cg, cb) = self.attributes.color;
        let has_color = (cr, cg, cb) != (0, 0, 0);

        let mut out = String::new();

        if has_size {
            out.push_str(&format!("<span style=\"font-size:{:.0}%;\">", size_pct));
        }
        if has_color {
            out.push_str(&format!(
                "<span style=\"color:rgb({},{},{});\">",
                cr, cg, cb
            ));
        }

        for i in 0..formats.len() {
            let fmt = formats[i];
            let text = self.run_text(&formats, i)?;

            if fmt.flags.contains(TextFlags::FORMULA) {
                out.push('$');
                out.push_str(text);
                out.push('$');
            } else {
                let mut seg = text.to_owned();
                if fmt.flags.contains(TextFlags::RED) {
                    seg = format!("<span style=\"color:red;\">{}</span>", seg);
                }
                if fmt.flags.contains(TextFlags::MONOSPACE) {
                    seg = format!("<code>{}</code>", seg);
                }
                if fmt.flags.contains(TextFlags::SUBSCRIPT) {
                    seg = format!("<sub>{}</sub>", seg);
                }
                if fmt.flags.contains(TextFlags::SUPERSCRIPT) {
                    seg = format!("<sup>{}</sup>", seg);
                }
                if fmt.flags.contains(TextFlags::STRIKETHROUGH) {
                    seg = format!("<del>{}</del>", seg);
                }
                if fmt.flags.contains(TextFlags::UNDERWAVE) {
                    seg = format!(
                        "<span style=\"text-decoration:underline wavy;\">{}</span>",
                        seg
                    );
                }
                if fmt.flags.contains(TextFlags::UNDERLINE) {
                    seg = format!("<span style=\"text-decoration:underline;\">{}</span>", seg);
                }
                if fmt.flags.contains(TextFlags::ITALIC) {
                    seg = format!("<em>{}</em>", seg);
                }
                if fmt.flags.contains(TextFlags::BOLD) {
                    seg = format!("<strong>{}</strong>", seg);
                }
                out.push_str(&seg);
            }
        }

        if has_color {
            out.push_str("</span>");
        }
        if has_size {
            out.push_str("</span>");
        }

        Ok(out)
    }
}

impl Renderer<Markdown, Universal> for Text {
    /// Render the `Text` element into a Markdown fragment and write it to stdout.
    ///
    /// Markdown has no native font-size or colour syntax; those fall back to
    /// inline HTML `<span>` wrappers (identical to the HTML renderer).
    ///
    /// Per-run [`TextFlags`]:
    /// | Flag           | Markdown output                                          |
    /// |----------------|----------------------------------------------------------|
    /// | `FORMULA`      | `$…$`  (CommonMark / GFM math)                           |
    /// | `BOLD`         | `**…**`                                                  |
    /// | `ITALIC`       | `_…_`                                                    |
    /// | `STRIKETHROUGH`| `~~…~~`                                                  |
    /// | `MONOSPACE`    | `` `…` ``                                                |
    /// | `UNDERLINE`    | `<u>…</u>`                                               |
    /// | `UNDERWAVE`    | `<span style="text-decoration:underline wavy;">…</span>` |
    /// | `SUPERSCRIPT`  | `<sup>…</sup>`                                           |
    /// | `SUBSCRIPT`    | `<sub>…</sub>`                                           |
    /// | `RED`          | `<span style="color:red;">…</span>`                      |
    fn render(&self) -> anyhow::Result<String> {
        let formats = self.parse_formats();

        let size_pct = self.attributes.font_size.ratio() * 100.0;
        let has_size = self.attributes.font_size != FontSize::Normal;

        let (cr, cg, cb) = self.attributes.color;
        let has_color = (cr, cg, cb) != (0, 0, 0);

        let mut out = String::new();

        if has_size {
            out.push_str(&format!("<span style=\"font-size:{:.0}%;\">", size_pct));
        }
        if has_color {
            out.push_str(&format!(
                "<span style=\"color:rgb({},{},{});\">",
                cr, cg, cb
            ));
        }

        for i in 0..formats.len() {
            let fmt = formats[i];
            let text = self.run_text(&formats, i)?;

            if fmt.flags.contains(TextFlags::FORMULA) {
                out.push('$');
                out.push_str(text);
                out.push('$');
            } else {
                let mut seg = text.to_owned();
                // HTML-only flags (no native Markdown equivalent).
                if fmt.flags.contains(TextFlags::RED) {
                    seg = format!("<span style=\"color:red;\">{}</span>", seg);
                }
                if fmt.flags.contains(TextFlags::SUBSCRIPT) {
                    seg = format!("<sub>{}</sub>", seg);
                }
                if fmt.flags.contains(TextFlags::SUPERSCRIPT) {
                    seg = format!("<sup>{}</sup>", seg);
                }
                if fmt.flags.contains(TextFlags::UNDERWAVE) {
                    seg = format!(
                        "<span style=\"text-decoration:underline wavy;\">{}</span>",
                        seg
                    );
                }
                if fmt.flags.contains(TextFlags::UNDERLINE) {
                    seg = format!("<u>{}</u>", seg);
                }
                // Native Markdown flags.
                if fmt.flags.contains(TextFlags::MONOSPACE) {
                    seg = format!("`{}`", seg);
                }
                if fmt.flags.contains(TextFlags::STRIKETHROUGH) {
                    seg = format!("~~{}~~", seg);
                }
                if fmt.flags.contains(TextFlags::ITALIC) {
                    seg = format!("_{}_", seg);
                }
                if fmt.flags.contains(TextFlags::BOLD) {
                    seg = format!("**{}**", seg);
                }
                out.push_str(&seg);
            }
        }

        if has_color {
            out.push_str("</span>");
        }
        if has_size {
            out.push_str("</span>");
        }

        Ok(out)
    }
}

impl Renderer<Proprietary, Universal> for Text {
    /// Render the `Text` element back into the proprietary delimiter format.
    ///
    /// This is the inverse of [`Text::parse`]: it reconstructs the
    /// `\[<flags>]{<content>}` markup that was used to create the element,
    /// making it suitable for round-tripping or storage.
    ///
    /// Plain runs (no flags) are emitted as raw text.  Formatted runs are
    /// wrapped in `\[<flags>]{…}`.
    ///
    /// Global [`TextAttributes`] that differ from their defaults are prefixed
    /// as proprietary markers before the run content:
    /// - `font_size` (non-Normal) → `\@size{<name>}` where `<name>` is one of
    ///   `tiny`, `script`, `footnote`, `small`, `large`, `xlarge`, `xxlarge`.
    /// - `color` (non-black)      → `\@color{r,g,b}`
    ///
    /// Flag letters (same as the parse syntax):
    ///
    /// | Letter | Flag           |
    /// |--------|----------------|
    /// | `b`    | BOLD           |
    /// | `i`    | ITALIC         |
    /// | `u`    | UNDERLINE      |
    /// | `w`    | UNDERWAVE      |
    /// | `d`    | STRIKETHROUGH  |
    /// | `s`    | SUPERSCRIPT    |
    /// | `x`    | SUBSCRIPT      |
    /// | `m`    | MONOSPACE      |
    /// | `f`    | FORMULA        |
    /// | `r`    | RED            |
    fn render(&self) -> anyhow::Result<String> {
        let formats = self.parse_formats();

        let mut out = String::new();

        // ── Global attribute markers ──────────────────────────────────────────
        if self.attributes.font_size != FontSize::Normal {
            let name = match self.attributes.font_size {
                FontSize::Tiny => "tiny",
                FontSize::Script => "script",
                FontSize::Footnote => "footnote",
                FontSize::Small => "small",
                FontSize::Normal => "normal",
                FontSize::Large => "large",
                FontSize::XLarge => "xlarge",
                FontSize::XXLarge => "xxlarge",
            };
            out.push_str(&format!("\\@size{{{}}}", name));
        }

        let (cr, cg, cb) = self.attributes.color;
        if (cr, cg, cb) != (0, 0, 0) {
            out.push_str(&format!("\\@color{{{},{},{}}}", cr, cg, cb));
        }

        // ── Per-run content ───────────────────────────────────────────────────
        for i in 0..formats.len() {
            let fmt = formats[i];
            let text = self.run_text(&formats, i)?;

            if fmt.flags.is_empty() {
                out.push_str(text);
            } else {
                let mut flags_str = String::new();
                if fmt.flags.contains(TextFlags::BOLD) {
                    flags_str.push('b');
                }
                if fmt.flags.contains(TextFlags::ITALIC) {
                    flags_str.push('i');
                }
                if fmt.flags.contains(TextFlags::UNDERLINE) {
                    flags_str.push('u');
                }
                if fmt.flags.contains(TextFlags::UNDERWAVE) {
                    flags_str.push('w');
                }
                if fmt.flags.contains(TextFlags::STRIKETHROUGH) {
                    flags_str.push('d');
                }
                if fmt.flags.contains(TextFlags::SUPERSCRIPT) {
                    flags_str.push('s');
                }
                if fmt.flags.contains(TextFlags::SUBSCRIPT) {
                    flags_str.push('x');
                }
                if fmt.flags.contains(TextFlags::MONOSPACE) {
                    flags_str.push('m');
                }
                if fmt.flags.contains(TextFlags::FORMULA) {
                    flags_str.push('f');
                }
                if fmt.flags.contains(TextFlags::RED) {
                    flags_str.push('r');
                }
                out.push_str(&format!("\\[{}]{{{}}}", flags_str, text));
            }
        }

        Ok(out)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::super::super::renderer::{Html, Latex, Markdown, Proprietary, Renderer};
    use super::*;

    // ── helpers ───────────────────────────────────────────────────────────────

    fn parse(s: &str) -> Text {
        Text::parse(s).expect("parse failed")
    }

    fn latex(s: &str) -> String {
        <Text as Renderer<Latex, Universal>>::render(&parse(s)).expect("latex render failed")
    }

    fn html(s: &str) -> String {
        <Text as Renderer<Html, Universal>>::render(&parse(s)).expect("html render failed")
    }

    fn md(s: &str) -> String {
        <Text as Renderer<Markdown, Universal>>::render(&parse(s)).expect("markdown render failed")
    }

    fn prop(s: &str) -> String {
        <Text as Renderer<Proprietary, Universal>>::render(&parse(s))
            .expect("proprietary render failed")
    }

    // ── TextFormat round-trip ─────────────────────────────────────────────────

    #[test]
    fn textformat_round_trip() {
        let fmt = TextFormat {
            language: 1,
            start: 42,
            flags: TextFlags::BOLD | TextFlags::ITALIC,
        };
        assert_eq!(TextFormat::from_bytes(&fmt.to_bytes()), fmt);
    }

    #[test]
    fn textformat_all_flags_round_trip() {
        let all = TextFlags::BOLD
            | TextFlags::ITALIC
            | TextFlags::UNDERLINE
            | TextFlags::UNDERWAVE
            | TextFlags::STRIKETHROUGH
            | TextFlags::SUPERSCRIPT
            | TextFlags::SUBSCRIPT
            | TextFlags::MONOSPACE
            | TextFlags::FORMULA
            | TextFlags::RED;
        let fmt = TextFormat {
            language: 0,
            start: 0,
            flags: all,
        };
        assert_eq!(TextFormat::from_bytes(&fmt.to_bytes()).flags, all);
    }

    // ── Parser ────────────────────────────────────────────────────────────────

    #[test]
    fn parse_plain_text() {
        let t = parse("hello world");
        let fmts = t.parse_formats();
        assert_eq!(fmts.len(), 1);
        assert_eq!(fmts[0].flags, TextFlags::empty());
        assert_eq!(&t.content, b"hello world");
    }

    #[test]
    fn parse_single_flag() {
        let t = parse(r"\[b]{bold}");
        let fmts = t.parse_formats();
        assert_eq!(fmts.len(), 1);
        assert!(fmts[0].flags.contains(TextFlags::BOLD));
        assert_eq!(&t.content, b"bold");
    }

    #[test]
    fn parse_combined_flags() {
        let t = parse(r"\[bi]{bold italic}");
        let fmts = t.parse_formats();
        assert_eq!(fmts.len(), 1);
        assert!(fmts[0].flags.contains(TextFlags::BOLD));
        assert!(fmts[0].flags.contains(TextFlags::ITALIC));
    }

    #[test]
    fn parse_multiple_runs() {
        let t = parse(r"Hello \[b]{World} and \[bi]{this}.");
        let fmts = t.parse_formats();
        assert_eq!(fmts[0].flags, TextFlags::empty()); // "Hello "
        assert_eq!(fmts[1].flags, TextFlags::BOLD); // "World"
        assert_eq!(fmts[2].flags, TextFlags::empty()); // " and "
        assert_eq!(fmts[3].flags, TextFlags::BOLD | TextFlags::ITALIC); // "this"
        assert_eq!(fmts[4].flags, TextFlags::empty()); // "."
    }

    #[test]
    fn parse_start_pointers_are_correct() {
        let t = parse(r"ab\[b]{cd}ef");
        let fmts = t.parse_formats();
        assert_eq!(fmts[0].start, 0); // "ab"
        assert_eq!(fmts[1].start, 2); // "cd"
        assert_eq!(fmts[2].start, 4); // "ef"
        assert_eq!(&t.content, b"abcdef");
    }

    #[test]
    fn parse_nested_braces_in_content() {
        let t = parse(r"\[f]{x^{2}}");
        let fmts = t.parse_formats();
        assert_eq!(fmts.len(), 1);
        assert!(fmts[0].flags.contains(TextFlags::FORMULA));
        assert_eq!(std::str::from_utf8(&t.content).unwrap(), "x^{2}");
    }

    #[test]
    fn parse_all_single_flags() {
        let cases: &[(&str, TextFlags)] = &[
            (r"\[b]{x}", TextFlags::BOLD),
            (r"\[i]{x}", TextFlags::ITALIC),
            (r"\[u]{x}", TextFlags::UNDERLINE),
            (r"\[w]{x}", TextFlags::UNDERWAVE),
            (r"\[d]{x}", TextFlags::STRIKETHROUGH),
            (r"\[s]{x}", TextFlags::SUPERSCRIPT),
            (r"\[x]{x}", TextFlags::SUBSCRIPT),
            (r"\[m]{x}", TextFlags::MONOSPACE),
            (r"\[f]{x}", TextFlags::FORMULA),
            (r"\[r]{x}", TextFlags::RED),
        ];
        for (src, expected) in cases {
            let fmts = parse(src).parse_formats();
            assert_eq!(fmts[0].flags, *expected, "failed for {src:?}");
        }
    }

    #[test]
    fn parse_empty_content() {
        let t = parse(r"\[b]{}");
        let fmts = t.parse_formats();
        assert_eq!(fmts.len(), 1);
        assert_eq!(fmts[0].flags, TextFlags::BOLD);
        assert!(t.content.is_empty());
    }

    #[test]
    fn parse_empty_input() {
        let t = parse("");
        assert!(t.formatting.is_empty());
        assert!(t.content.is_empty());
    }

    #[test]
    fn parse_unknown_flag_errors() {
        assert!(Text::parse(r"\[z]{x}").is_err());
    }

    #[test]
    fn parse_missing_close_bracket_errors() {
        assert!(Text::parse(r"\[b{x}").is_err());
    }

    #[test]
    fn parse_unclosed_brace_errors() {
        assert!(Text::parse(r"\[b]{unclosed").is_err());
    }

    #[test]
    fn parse_from_str_trait() {
        let t: Text = r"\[b]{hi}".parse().unwrap();
        assert_eq!(t.parse_formats()[0].flags, TextFlags::BOLD);
    }

    // ── LaTeX renderer ────────────────────────────────────────────────────────

    #[test]
    fn latex_plain_text() {
        assert_eq!(latex("hello"), "hello");
    }

    #[test]
    fn latex_bold() {
        assert_eq!(latex(r"\[b]{World}"), r"\textbf{World}");
    }

    #[test]
    fn latex_italic() {
        assert_eq!(latex(r"\[i]{slant}"), r"\textit{slant}");
    }

    #[test]
    fn latex_bold_italic() {
        assert_eq!(latex(r"\[bi]{both}"), r"\textbf{\textit{both}}");
    }

    #[test]
    fn latex_underline() {
        assert_eq!(latex(r"\[u]{line}"), r"\underline{line}");
    }

    #[test]
    fn latex_underwave() {
        assert_eq!(latex(r"\[w]{wave}"), r"\uwave{wave}");
    }

    #[test]
    fn latex_strikethrough() {
        assert_eq!(latex(r"\[d]{del}"), r"\sout{del}");
    }

    #[test]
    fn latex_superscript() {
        assert_eq!(latex(r"\[s]{up}"), r"\textsuperscript{up}");
    }

    #[test]
    fn latex_subscript() {
        assert_eq!(latex(r"\[x]{down}"), r"\textsubscript{down}");
    }

    #[test]
    fn latex_monospace() {
        assert_eq!(latex(r"\[m]{code}"), r"\texttt{code}");
    }

    #[test]
    fn latex_formula() {
        assert_eq!(latex(r"\[f]{E=mc^2}"), "$E=mc^2$");
    }

    #[test]
    fn latex_red() {
        assert_eq!(latex(r"\[r]{red}"), r"\textcolor{red}{red}");
    }

    #[test]
    fn latex_mixed_runs() {
        assert_eq!(latex(r"Hello \[b]{World}!"), r"Hello \textbf{World}!");
    }

    #[test]
    fn latex_font_size_large() {
        let mut t = parse(r"\[b]{big}");
        t.attributes.font_size = FontSize::Large;
        assert_eq!(
            <Text as Renderer<Latex, Universal>>::render(&t).unwrap(),
            r"{\large \textbf{big}}"
        );
    }

    #[test]
    fn latex_font_size_tiny() {
        let mut t = parse("tiny");
        t.attributes.font_size = FontSize::Tiny;
        assert_eq!(
            <Text as Renderer<Latex, Universal>>::render(&t).unwrap(),
            r"{\tiny tiny}"
        );
    }

    #[test]
    fn latex_color_attribute() {
        let mut t = parse("red text");
        t.attributes.color = (255, 0, 0);
        assert_eq!(
            <Text as Renderer<Latex, Universal>>::render(&t).unwrap(),
            r"\textcolor[RGB]{255,0,0}{red text}"
        );
    }

    #[test]
    fn latex_size_and_color() {
        let mut t = parse("both");
        t.attributes.font_size = FontSize::Small;
        t.attributes.color = (0, 128, 255);
        assert_eq!(
            <Text as Renderer<Latex, Universal>>::render(&t).unwrap(),
            r"{\small \textcolor[RGB]{0,128,255}{both}}"
        );
    }

    #[test]
    fn latex_formula_ignores_other_flags() {
        // FORMULA takes priority; bold is ignored inside a formula run.
        assert_eq!(latex(r"\[fb]{x^2}"), "$x^2$");
    }

    // ── HTML renderer ─────────────────────────────────────────────────────────

    #[test]
    fn html_plain_text() {
        assert_eq!(html("hello"), "hello");
    }

    #[test]
    fn html_bold() {
        assert_eq!(html(r"\[b]{World}"), "<strong>World</strong>");
    }

    #[test]
    fn html_italic() {
        assert_eq!(html(r"\[i]{slant}"), "<em>slant</em>");
    }

    #[test]
    fn html_bold_italic() {
        assert_eq!(html(r"\[bi]{both}"), "<strong><em>both</em></strong>");
    }

    #[test]
    fn html_underline() {
        assert_eq!(
            html(r"\[u]{line}"),
            r#"<span style="text-decoration:underline;">line</span>"#
        );
    }

    #[test]
    fn html_underwave() {
        assert_eq!(
            html(r"\[w]{wave}"),
            r#"<span style="text-decoration:underline wavy;">wave</span>"#
        );
    }

    #[test]
    fn html_strikethrough() {
        assert_eq!(html(r"\[d]{del}"), "<del>del</del>");
    }

    #[test]
    fn html_superscript() {
        assert_eq!(html(r"\[s]{up}"), "<sup>up</sup>");
    }

    #[test]
    fn html_subscript() {
        assert_eq!(html(r"\[x]{down}"), "<sub>down</sub>");
    }

    #[test]
    fn html_monospace() {
        assert_eq!(html(r"\[m]{code}"), "<code>code</code>");
    }

    #[test]
    fn html_formula() {
        assert_eq!(html(r"\[f]{E=mc^2}"), "$E=mc^2$");
    }

    #[test]
    fn html_red() {
        assert_eq!(html(r"\[r]{red}"), r#"<span style="color:red;">red</span>"#);
    }

    #[test]
    fn html_mixed_runs() {
        assert_eq!(html(r"Hello \[b]{World}!"), "Hello <strong>World</strong>!");
    }

    #[test]
    fn html_font_size_large() {
        let mut t = parse(r"\[i]{big}");
        t.attributes.font_size = FontSize::Large;
        assert_eq!(
            <Text as Renderer<Html, Universal>>::render(&t).unwrap(),
            r#"<span style="font-size:120%;"><em>big</em></span>"#
        );
    }

    #[test]
    fn html_color_attribute() {
        let mut t = parse("coloured");
        t.attributes.color = (255, 128, 0);
        assert_eq!(
            <Text as Renderer<Html, Universal>>::render(&t).unwrap(),
            r#"<span style="color:rgb(255,128,0);">coloured</span>"#
        );
    }

    #[test]
    fn html_black_color_no_wrapper() {
        let mut t = parse("black");
        t.attributes.color = (0, 0, 0);
        assert_eq!(
            <Text as Renderer<Html, Universal>>::render(&t).unwrap(),
            "black"
        );
    }

    #[test]
    fn html_normal_size_no_wrapper() {
        let t = parse("normal");
        assert_eq!(
            <Text as Renderer<Html, Universal>>::render(&t).unwrap(),
            "normal"
        );
    }

    #[test]
    fn html_formula_ignores_other_flags() {
        assert_eq!(html(r"\[fb]{x^2}"), "$x^2$");
    }

    // ── Markdown renderer ─────────────────────────────────────────────────────

    #[test]
    fn md_plain_text() {
        assert_eq!(md("hello"), "hello");
    }

    #[test]
    fn md_bold() {
        assert_eq!(md(r"\[b]{World}"), "**World**");
    }

    #[test]
    fn md_italic() {
        assert_eq!(md(r"\[i]{slant}"), "_slant_");
    }

    #[test]
    fn md_bold_italic() {
        assert_eq!(md(r"\[bi]{both}"), "**_both_**");
    }

    #[test]
    fn md_strikethrough() {
        assert_eq!(md(r"\[d]{del}"), "~~del~~");
    }

    #[test]
    fn md_monospace() {
        assert_eq!(md(r"\[m]{code}"), "`code`");
    }

    #[test]
    fn md_formula() {
        assert_eq!(md(r"\[f]{E=mc^2}"), "$E=mc^2$");
    }

    #[test]
    fn md_underline() {
        assert_eq!(md(r"\[u]{line}"), "<u>line</u>");
    }

    #[test]
    fn md_underwave() {
        assert_eq!(
            md(r"\[w]{wave}"),
            r#"<span style="text-decoration:underline wavy;">wave</span>"#
        );
    }

    #[test]
    fn md_superscript() {
        assert_eq!(md(r"\[s]{up}"), "<sup>up</sup>");
    }

    #[test]
    fn md_subscript() {
        assert_eq!(md(r"\[x]{down}"), "<sub>down</sub>");
    }

    #[test]
    fn md_red() {
        assert_eq!(md(r"\[r]{red}"), r#"<span style="color:red;">red</span>"#);
    }

    #[test]
    fn md_mixed_runs() {
        assert_eq!(md(r"Hello \[b]{World}!"), "Hello **World**!");
    }

    #[test]
    fn md_font_size_large() {
        let mut t = parse(r"\[b]{big}");
        t.attributes.font_size = FontSize::Large;
        assert_eq!(
            <Text as Renderer<Markdown, Universal>>::render(&t).unwrap(),
            r#"<span style="font-size:120%;">**big**</span>"#
        );
    }

    #[test]
    fn md_color_attribute() {
        let mut t = parse("coloured");
        t.attributes.color = (255, 0, 0);
        assert_eq!(
            <Text as Renderer<Markdown, Universal>>::render(&t).unwrap(),
            r#"<span style="color:rgb(255,0,0);">coloured</span>"#
        );
    }

    #[test]
    fn md_formula_ignores_other_flags() {
        assert_eq!(md(r"\[fb]{x^2}"), "$x^2$");
    }

    // ── Proprietary renderer ───────────────────────────────────────────────────

    #[test]
    fn prop_plain_text() {
        assert_eq!(prop("hello"), "hello");
    }

    #[test]
    fn prop_bold() {
        assert_eq!(prop(r"\[b]{World}"), r"\[b]{World}");
    }

    #[test]
    fn prop_italic() {
        assert_eq!(prop(r"\[i]{slant}"), r"\[i]{slant}");
    }

    #[test]
    fn prop_bold_italic() {
        assert_eq!(prop(r"\[bi]{both}"), r"\[bi]{both}");
    }

    #[test]
    fn prop_all_flags() {
        // Flags are always emitted in canonical order: b i u w d s x m f r.
        assert_eq!(prop(r"\[biuwdsxmfr]{all}"), r"\[biuwdsxmfr]{all}");
    }

    #[test]
    fn prop_mixed_runs() {
        assert_eq!(prop(r"Hello \[b]{World}!"), r"Hello \[b]{World}!");
    }

    #[test]
    fn prop_round_trip() {
        // Parsing and re-serialising should be a no-op.
        let src = r"\[b]{bold} and \[i]{italic} and plain.";
        assert_eq!(prop(src), src);
    }

    #[test]
    fn prop_size_attribute() {
        let mut t = parse("text");
        t.attributes.font_size = FontSize::Large;
        assert_eq!(
            <Text as Renderer<Proprietary, Universal>>::render(&t).unwrap(),
            r"\@size{large}text"
        );
    }

    #[test]
    fn prop_normal_size_no_marker() {
        let t = parse("text");
        assert_eq!(
            <Text as Renderer<Proprietary, Universal>>::render(&t).unwrap(),
            "text"
        );
    }

    #[test]
    fn prop_color_attribute() {
        let mut t = parse("text");
        t.attributes.color = (255, 0, 128);
        assert_eq!(
            <Text as Renderer<Proprietary, Universal>>::render(&t).unwrap(),
            r"\@color{255,0,128}text"
        );
    }

    #[test]
    fn prop_black_color_no_marker() {
        let mut t = parse("text");
        t.attributes.color = (0, 0, 0);
        assert_eq!(
            <Text as Renderer<Proprietary, Universal>>::render(&t).unwrap(),
            "text"
        );
    }

    #[test]
    fn prop_size_and_color_attributes() {
        let mut t = parse(r"\[b]{hi}");
        t.attributes.font_size = FontSize::Small;
        t.attributes.color = (0, 128, 255);
        assert_eq!(
            <Text as Renderer<Proprietary, Universal>>::render(&t).unwrap(),
            r"\@size{small}\@color{0,128,255}\[b]{hi}"
        );
    }

    #[test]
    fn prop_underline() {
        assert_eq!(prop(r"\[u]{line}"), r"\[u]{line}");
    }

    #[test]
    fn prop_underwave() {
        assert_eq!(prop(r"\[w]{wave}"), r"\[w]{wave}");
    }

    #[test]
    fn prop_strikethrough() {
        assert_eq!(prop(r"\[d]{del}"), r"\[d]{del}");
    }

    #[test]
    fn prop_superscript() {
        assert_eq!(prop(r"\[s]{up}"), r"\[s]{up}");
    }

    #[test]
    fn prop_subscript() {
        assert_eq!(prop(r"\[x]{down}"), r"\[x]{down}");
    }

    #[test]
    fn prop_monospace() {
        assert_eq!(prop(r"\[m]{code}"), r"\[m]{code}");
    }

    #[test]
    fn prop_formula() {
        assert_eq!(prop(r"\[f]{E=mc^2}"), r"\[f]{E=mc^2}");
    }

    #[test]
    fn prop_red() {
        assert_eq!(prop(r"\[r]{red}"), r"\[r]{red}");
    }

    #[test]
    fn prop_formula_with_other_flags_round_trips() {
        // The parser accepts \[fb] and stores both flags; the proprietary
        // renderer faithfully re-emits whatever flags are recorded.
        assert_eq!(prop(r"\[fb]{x^2}"), r"\[bf]{x^2}");
    }

    #[test]
    fn prop_nested_braces_in_formula() {
        assert_eq!(prop(r"\[f]{x^{2}}"), r"\[f]{x^{2}}");
    }

    #[test]
    fn prop_empty_content() {
        assert_eq!(prop(r"\[b]{}"), r"\[b]{}");
    }

    #[test]
    fn prop_empty_input() {
        assert_eq!(prop(""), "");
    }

    #[test]
    fn prop_size_tiny() {
        let mut t = parse("x");
        t.attributes.font_size = FontSize::Tiny;
        assert_eq!(
            <Text as Renderer<Proprietary, Universal>>::render(&t).unwrap(),
            r"\@size{tiny}x"
        );
    }

    #[test]
    fn prop_size_script() {
        let mut t = parse("x");
        t.attributes.font_size = FontSize::Script;
        assert_eq!(
            <Text as Renderer<Proprietary, Universal>>::render(&t).unwrap(),
            r"\@size{script}x"
        );
    }

    #[test]
    fn prop_size_footnote() {
        let mut t = parse("x");
        t.attributes.font_size = FontSize::Footnote;
        assert_eq!(
            <Text as Renderer<Proprietary, Universal>>::render(&t).unwrap(),
            r"\@size{footnote}x"
        );
    }

    #[test]
    fn prop_size_xlarge() {
        let mut t = parse("x");
        t.attributes.font_size = FontSize::XLarge;
        assert_eq!(
            <Text as Renderer<Proprietary, Universal>>::render(&t).unwrap(),
            r"\@size{xlarge}x"
        );
    }

    #[test]
    fn prop_size_xxlarge() {
        let mut t = parse("x");
        t.attributes.font_size = FontSize::XXLarge;
        assert_eq!(
            <Text as Renderer<Proprietary, Universal>>::render(&t).unwrap(),
            r"\@size{xxlarge}x"
        );
    }

    #[test]
    fn prop_multiple_formatted_runs() {
        let src = r"\[b]{bold} middle \[i]{italic}";
        assert_eq!(prop(src), src);
    }

    #[test]
    fn prop_potpourri() {
        let mut t = parse(
            r"In this section, we will introduce \[b]{important} concepts about \[i]{Faraday's Electromagnetic Induction} and \[f]{\int_a^b \mathbf{E} \cdot d\mathbf{l}}.",
        );
        t.attributes.font_size = FontSize::Large;
        t.attributes.color = (0, 128, 255);
        assert_eq!(
            <Text as Renderer<Proprietary, Universal>>::render(&t).unwrap(),
            r"\@size{large}\@color{0,128,255}In this section, we will introduce \[b]{important} concepts about \[i]{Faraday's Electromagnetic Induction} and \[f]{\int_a^b \mathbf{E} \cdot d\mathbf{l}}."
        );
    }

    #[test]
    fn tex_melt_potpourri() {
        let mut t = parse(
            r"In this section, we will introduce \[b]{important} concepts about \[i]{Faraday's Electromagnetic Induction} and \[f]{\int_a^b \mathbf{E} \cdot d\mathbf{l}}.",
        );
        t.attributes.font_size = FontSize::Large;
        t.attributes.color = (0, 128, 255);
        assert_eq!(
            <Text as Renderer<Latex, Universal>>::render(&t).unwrap(),
            r#"{\large \textcolor[RGB]{0,128,255}{In this section, we will introduce \textbf{important} concepts about \textit{Faraday's Electromagnetic Induction} and $\int_a^b \mathbf{E} \cdot d\mathbf{l}$.}}"#
        );
    }
}
