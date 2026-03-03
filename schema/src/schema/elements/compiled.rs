use crate::{DocumentClass, LatexBuilder, XeLaTeX, schema::utils::{compile_latex_to_svg, compile_svg_to_png}};

pub struct CompiledGraph {
    pub tex_code: String,
    pub svg_content: String,
    pub png_content: Vec<u8>,
}

impl CompiledGraph {
    pub fn new(tex_code: String, svg_content: String, png_content: Vec<u8>) -> Self {
        Self {
            tex_code,
            svg_content,
            png_content,
        }
    }

    pub async fn from_latex(latex_content: String) -> anyhow::Result<Self> {
        let svg_content = compile_latex_to_svg::<XeLaTeX>(latex_content.as_str()).await?;
        let png_content = compile_svg_to_png(&svg_content).await?;

        Ok(Self::new(latex_content, svg_content, png_content))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_compiled_graph() {
        let latex = r"Hello, world! $x^2$ \chemfig{A-B-C}";

        let compiled = CompiledGraph::from_latex(latex.to_string()).await.unwrap();
        assert!(compiled.tex_code.contains("Hello, world!"));
        assert!(compiled.svg_content.contains("<svg"));
        assert!(!compiled.png_content.is_empty());
    }
}