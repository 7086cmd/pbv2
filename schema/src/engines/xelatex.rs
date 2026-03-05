use crate::engines::Engine;
use anyhow::Result;
use std::path::PathBuf;

pub struct XeLaTeX;

#[async_trait::async_trait]
impl Engine for XeLaTeX {
    fn name() -> &'static str {
        "xelatex"
    }

    async fn compile_once(addr: PathBuf) -> Result<PathBuf> {
        let output = tokio::process::Command::new("xelatex")
            .current_dir(addr.parent().unwrap())
            .arg("-interaction=nonstopmode")
            .arg("--shell-escape")
            .arg(addr.to_str().unwrap())
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "XeLaTeX compilation failed: {}",
                String::from_utf8_lossy(&output.stdout) // Note that XeLaTeX writes errors to stdout, not stderr
            ));
        }

        let mut output_path = addr.clone();
        output_path.set_extension("pdf");
        Ok(output_path)
    }
}
