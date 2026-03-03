use std::path::PathBuf;
use anyhow::Result;
use sha1::{Sha1, Digest};

use crate::{Engine, LatexBuilder};

pub async fn compile_latex_to_svg<E: Engine + 'static>(latex: &str) -> Result<String> {
    let temporary_dir = temp_dir::TempDir::new()?;
    let folder = temporary_dir.path().to_path_buf();

    let (tx, rx) = tokio::sync::oneshot::channel();

    // Step 1. Store file into `<tempdir>/folder/timestamp_hash[:6].tex`, and the hash is sha-1
    let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_millis();
    let mut hasher = Sha1::new();
    hasher.update(timestamp.to_be_bytes());
    let hashed_hex = hex::encode(hasher.finalize());
    let filename_no_prefix = hashed_hex[..6].to_string();
    let filepath = folder.join(filename_no_prefix + ".tex");
    let mut latex_content = LatexBuilder::new(crate::DocumentClass::Standalone);
    latex_content.add_content(latex.to_string());
    tokio::fs::write(&filepath, latex_content.build()).await?;
    println!("LaTeX file written to: {}", filepath.to_str().unwrap());
    println!("LaTeX content:\n{}", latex_content.build());

    // Step 2. Compile the .tex file into .pdf and then convert to .svg using `E::` methods
    tokio::spawn(async move {
        let result: Result<PathBuf> = async {
            let result_pdf = E::compile(filepath, false).await?;
            let result_svg = result_pdf.with_extension("svg");
            let inkscape_output = tokio::process::Command::new("inkscape")
                .current_dir(&folder)
                .arg(result_pdf.to_str().unwrap())
                .arg("--export-type=svg")
                .arg("--export-plain-svg")
                .arg("-o")
                .arg(result_svg.to_str().unwrap())
                .output().await?;
            if !inkscape_output.status.success() {
                return Err(anyhow::anyhow!(
                    "Inkscape conversion failed: {}",
                    String::from_utf8_lossy(&inkscape_output.stderr)
                ));
            }
            Ok(result_svg)
        }.await;
        let _ = tx.send(result);
    });
    let result_svg = rx.await??;
    let svg_content = tokio::fs::read_to_string(&result_svg).await?;

    println!("SVG content generated:\n{}", svg_content);
    temporary_dir.cleanup()?;
    Ok(svg_content)
}

pub async fn compile_svg_to_png(svg_content: &str) -> Result<Vec<u8>> {
    // 1. Parse the SVG
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_data(svg_content.as_bytes(), &opt).unwrap();

    let pixmap_size = tree.size().to_int_size();
    let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();

    // 2. Render the SVG to the Pixmap (buffer)
    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());

    pixmap.encode_png().map_err(|e| anyhow::anyhow!("Failed to encode PNG: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_compile_latex_to_svg() {
        let latex = r"Hello, world! $x^2$

\begin{tikzpicture}
  \draw[wall] (0,0) -- (4,0); % Draw a horizontal wall
  \draw[wall] (4,0) -- (4,4); % Draw a vertical wall
\end{tikzpicture}

\chemfig{*6(=-=(-CH_2OH)-=-)}";
        let svg = compile_latex_to_svg::<crate::XeLaTeX>(latex).await.unwrap();
        assert!(svg.contains("<svg"));
    }
}