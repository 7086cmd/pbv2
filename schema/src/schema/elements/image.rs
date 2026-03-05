use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

use crate::CompiledGraph;
use crate::schema::renderer::{Html, Latex, Markdown, Renderer, Universal};

/// In LaTeX, we use `\\begin{writebase64}{filename} ... \\end{writebase64}` to
/// embed binary data (from the `base64` LaTeX package), which writes the decoded
/// bytes to `filename` on disk.  We then use `\\includegraphics` to pull that
/// file into the document.
///
/// In HTML / Markdown, the `data:` URI scheme is used so the image is fully
/// self-contained without any external file dependency.
#[derive(Clone, Debug)]
pub struct BinaryImage {
    pub buffer: Vec<u8>,
    pub format: ImageFormat,
    /// File stem used for the LaTeX `writebase64` target; auto-generated from
    /// the format when `None`.
    pub filename: Option<String>,
    /// Width as a fraction of `\textwidth`, e.g. `0.5` → `width=0.5\textwidth`.
    /// When `None` no width option is emitted (LaTeX natural size).
    pub width_ratio: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ImageFormat {
    Png,
    Jpeg,
}

#[derive(Clone, Debug)]
pub struct PdfSvgImage {
    pub pdf_buffer: Vec<u8>,
    pub svg_content: String,
    pub width_ratio: Option<f64>,
}

impl ImageFormat {
    /// MIME sub-type string, e.g. `"png"` or `"jpeg"`.
    pub fn mime(&self) -> &'static str {
        match self {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpeg",
        }
    }

    /// Typical file extension without the dot.
    pub fn extension(&self) -> &'static str {
        match self {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpg",
        }
    }
}

#[derive(Clone, Debug)]
pub enum Image {
    Binary(BinaryImage),
    Url(String),
    Latex(CompiledGraph),
    PdfSvg(PdfSvgImage),
}

// ── LaTeX ─────────────────────────────────────────────────────────────────────

impl Renderer<Latex, Universal> for Image {
    /// Render the image as a LaTeX fragment.
    ///
    /// | Variant            | Output                                              |
    /// |--------------------|-----------------------------------------------------|
    /// | `Binary`           | `\begin{writebase64}{file}\n<base64>\n\end{writebase64}\n\includegraphics[…]{file}` |
    /// | `Url`              | *(not supported — returns an error)*               |
    /// | `Latex(compiled)`  | The raw `tex_code` from the [`CompiledGraph`]       |
    fn render(&self) -> anyhow::Result<String> {
        match self {
            Image::Binary(img) => {
                let filename = img
                    .filename
                    .clone()
                    .unwrap_or_else(|| format!("image.{}", img.format.extension()));
                let b64 = BASE64.encode(&img.buffer);

                let width_opt = match img.width_ratio {
                    Some(r) => format!("[width={:.6}\\textwidth]", r),
                    None => String::new(),
                };

                let centering = img.width_ratio.unwrap_or(1f64).clamp(0.0, 1.0) > 0.5;

                Ok(format!(
                    "\\begin{{writebase64}}{{{filename}}}\n{b64}\n\\end{{writebase64}}\n\\begin{{figure}}[H]\n\\{}\n\\includegraphics{width_opt}{{{filename}}}\n\\end{{figure}}",
                    if centering { "centering" } else { "flushright" },
                ))
            }
            Image::Url(_) => {
                anyhow::bail!("URL images are not supported in the LaTeX renderer");
            }
            Image::Latex(graph) => Ok(graph.tex_code.clone()),
            Image::PdfSvg(payload) => {
                let width_opt = match payload.width_ratio {
                    Some(r) => format!("[width={:.6}\\textwidth]", r),
                    None => String::new(),
                };
                let filename = "image.pdf"; // The `writebase64` filename doesn't matter much here.
                let b64 = BASE64.encode(&payload.pdf_buffer);
                let centering = payload.width_ratio.unwrap_or(1f64).clamp(0.0, 1.0) > 0.5;
                Ok(format!(
                    "\\begin{{writebase64}}{{{filename}}}\n{b64}\n\\end{{writebase64}}\n\\begin{{figure}}[H]\n\\{}\n\\includegraphics{width_opt}{{{filename}}}\n\\end{{figure}}",
                    if centering { "centering" } else { "flushright" },
                ))
            }
        }
    }
}

// ── HTML ──────────────────────────────────────────────────────────────────────

impl Renderer<Html, Universal> for Image {
    /// Render the image as an HTML `<img>` element.
    ///
    /// | Variant           | `src` attribute                                      |
    /// |-------------------|------------------------------------------------------|
    /// | `Binary`          | `data:image/{mime};base64,{base64}`                  |
    /// | `Url`             | The URL string verbatim                              |
    /// | `Latex(compiled)` | `data:image/png;base64,{png_content}` from the graph |
    fn render(&self) -> anyhow::Result<String> {
        let src = match self {
            Image::Binary(img) => {
                let b64 = BASE64.encode(&img.buffer);
                format!("data:image/{};base64,{}", img.format.mime(), b64)
            }
            Image::Url(url) => url.clone(),
            Image::Latex(graph) => {
                let b64 = BASE64.encode(&graph.png_content);
                format!("data:image/png;base64,{}", b64)
            }
            Image::PdfSvg(payload) => {
                let b64 = BASE64.encode(payload.svg_content.as_bytes());
                format!("data:image/svg+xml;base64,{}", b64)
            }
        };

        let width_style = if let Image::Binary(img) = self {
            img.width_ratio
                .map(|r| format!(" style=\"width:{:.4}%;\"", r * 100.0))
                .unwrap_or_default()
        } else {
            String::new()
        };

        Ok(format!("<img src=\"{src}\"{width_style}>"))
    }
}

// ── Markdown ──────────────────────────────────────────────────────────────────

impl Renderer<Markdown, Universal> for Image {
    /// Render the image as Markdown.
    ///
    /// | Variant           | Output                                               |
    /// |-------------------|------------------------------------------------------|
    /// | `Binary`          | `<img>` tag with a `data:` URI (identical to HTML)   |
    /// | `Url`             | `![]({url})`                                        |
    /// | `Latex(compiled)` | `<img>` tag with a PNG `data:` URI                   |
    ///
    /// `Binary` and `Latex` variants use an inline `<img>` tag because
    /// standard Markdown image syntax (`![]()`) does not support `data:` URIs
    /// portably across all renderers, and the tag also carries the width style.
    fn render(&self) -> anyhow::Result<String> {
        match self {
            Image::Url(url) => Ok(format!("![]({})", url)),
            other => <Image as Renderer<Html, Universal>>::render(other),
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::renderer::{Html, Latex, Markdown, Renderer, Universal};

    fn png_image(bytes: Vec<u8>) -> Image {
        Image::Binary(BinaryImage {
            buffer: bytes,
            format: ImageFormat::Png,
            filename: None,
            width_ratio: None,
        })
    }

    fn png_image_named(bytes: Vec<u8>, name: &str, ratio: f64) -> Image {
        Image::Binary(BinaryImage {
            buffer: bytes,
            format: ImageFormat::Png,
            filename: Some(name.to_owned()),
            width_ratio: Some(ratio),
        })
    }

    // ── ImageFormat helpers ───────────────────────────────────────────────────

    #[test]
    fn format_mime() {
        assert_eq!(ImageFormat::Png.mime(), "png");
        assert_eq!(ImageFormat::Jpeg.mime(), "jpeg");
    }

    #[test]
    fn format_extension() {
        assert_eq!(ImageFormat::Png.extension(), "png");
        assert_eq!(ImageFormat::Jpeg.extension(), "jpg");
    }

    // ── LaTeX ─────────────────────────────────────────────────────────────────

    #[test]
    fn latex_binary_contains_writebase64() {
        let img = png_image(vec![0u8, 1, 2, 3]);
        let out = <Image as Renderer<Latex, Universal>>::render(&img).unwrap();
        assert!(out.contains("\\begin{writebase64}"));
        assert!(out.contains("\\end{writebase64}"));
        assert!(out.contains("\\includegraphics{image.png}"));
    }

    #[test]
    fn latex_binary_named_with_width() {
        let img = png_image_named(vec![0xDE, 0xAD], "fig1.png", 0.5);
        let out = <Image as Renderer<Latex, Universal>>::render(&img).unwrap();
        assert!(out.contains("\\begin{writebase64}{fig1.png}"));
        assert!(out.contains("\\includegraphics[width=0.500000\\textwidth]{fig1.png}"));
        assert!(out.contains("\\begin{figure}[H]"));
    }

    #[test]
    fn latex_binary_base64_roundtrips() {
        let original = b"hello image".to_vec();
        let img = png_image(original.clone());
        let out = <Image as Renderer<Latex, Universal>>::render(&img).unwrap();
        // Extract the base64 line (second line).
        let b64_line = out.lines().nth(1).unwrap();
        let decoded = BASE64.decode(b64_line).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn latex_url_returns_error() {
        let img = Image::Url("https://example.com/img.png".to_owned());
        assert!(<Image as Renderer<Latex, Universal>>::render(&img).is_err());
    }

    #[test]
    fn latex_compiled_graph_emits_tex_code() {
        let graph = CompiledGraph::new("\\tikz …".to_owned(), String::new(), vec![]);
        let img = Image::Latex(graph);
        let out = <Image as Renderer<Latex, Universal>>::render(&img).unwrap();
        assert_eq!(out, "\\tikz …");
    }

    // ── HTML ──────────────────────────────────────────────────────────────────

    #[test]
    fn html_binary_data_uri() {
        let img = png_image(vec![1, 2, 3]);
        let out = <Image as Renderer<Html, Universal>>::render(&img).unwrap();
        assert!(out.starts_with("<img"));
        assert!(out.contains("data:image/png;base64,"));
    }

    #[test]
    fn html_binary_jpeg_mime() {
        let img = Image::Binary(BinaryImage {
            buffer: vec![0xFF, 0xD8],
            format: ImageFormat::Jpeg,
            filename: None,
            width_ratio: None,
        });
        let out = <Image as Renderer<Html, Universal>>::render(&img).unwrap();
        assert!(out.contains("data:image/jpeg;base64,"));
    }

    #[test]
    fn html_binary_width_style() {
        let img = png_image_named(vec![0], "x.png", 0.75);
        let out = <Image as Renderer<Html, Universal>>::render(&img).unwrap();
        assert!(out.contains("style=\"width:75.0000%;\""));
    }

    #[test]
    fn html_url() {
        let img = Image::Url("https://example.com/photo.jpg".to_owned());
        let out = <Image as Renderer<Html, Universal>>::render(&img).unwrap();
        assert_eq!(out, "<img src=\"https://example.com/photo.jpg\">");
    }

    #[test]
    fn html_compiled_graph_uses_png() {
        let png_bytes = vec![137, 80, 78, 71]; // PNG magic bytes
        let graph = CompiledGraph::new(String::new(), String::new(), png_bytes.clone());
        let img = Image::Latex(graph);
        let out = <Image as Renderer<Html, Universal>>::render(&img).unwrap();
        let expected_b64 = BASE64.encode(&png_bytes);
        assert!(out.contains(&format!("data:image/png;base64,{}", expected_b64)));
    }

    // ── Markdown ──────────────────────────────────────────────────────────────

    #[test]
    fn md_url_uses_markdown_syntax() {
        let img = Image::Url("https://example.com/img.png".to_owned());
        let out = <Image as Renderer<Markdown, Universal>>::render(&img).unwrap();
        assert_eq!(out, "![](https://example.com/img.png)");
    }

    #[test]
    fn md_binary_falls_back_to_html_img_tag() {
        let img = png_image(vec![1, 2, 3]);
        let out = <Image as Renderer<Markdown, Universal>>::render(&img).unwrap();
        assert!(out.starts_with("<img"));
        assert!(out.contains("data:image/png;base64,"));
    }

    #[test]
    fn md_compiled_graph_falls_back_to_html_img_tag() {
        let graph = CompiledGraph::new(String::new(), String::new(), vec![0, 1]);
        let img = Image::Latex(graph);
        let out = <Image as Renderer<Markdown, Universal>>::render(&img).unwrap();
        assert!(out.starts_with("<img"));
        assert!(out.contains("data:image/png;base64,"));
    }
}
