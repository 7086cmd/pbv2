use anyhow::Result;
use std::{path::PathBuf, process::Command};

mod builtin;
mod xelatex;

pub use builtin::BuiltinEngine;
pub use xelatex::XeLaTeX;

#[async_trait::async_trait]
pub trait Engine: Send + Sync {
    fn name() -> &'static str;

    /// Executes a single compilation of the LaTeX document at the given path. This is used internally by the `compile` method to perform multiple compilations as needed.
    async fn compile_once(addr: PathBuf) -> Result<PathBuf>;

    /// Compiles the LaTeX document at the given path. If `compile_bib` is true, it will also run BibTeX.
    ///
    /// The order of compilation is as follows:
    /// 1. Compile the LaTeX document once to generate auxiliary files.
    /// 2. If `compile_bib` is true, run BibTeX on the generated `.aux` file. Then compile the LaTeX document twice more to ensure all references are updated.
    /// 3. If `compile_bib` is false, compile the LaTeX document once more to ensure all references are updated.
    async fn compile(addr: PathBuf, compile_bib: bool) -> Result<PathBuf> {
        let mut output_path = Self::compile_once(addr.clone()).await?;

        if compile_bib {
            let bib_output = Command::new("bibtex")
                .current_dir(addr.parent().unwrap())
                .arg(output_path.with_extension("aux").to_str().unwrap())
                .output()?;

            if !bib_output.status.success() {
                return Err(anyhow::anyhow!(
                    "BibTeX compilation failed: {}",
                    String::from_utf8_lossy(&bib_output.stderr)
                ));
            }

            output_path = Self::compile_once(addr.clone()).await?;
            Self::compile_once(addr.clone()).await?;
        }

        Self::compile_once(addr.clone()).await?;

        Ok(output_path)
    }
}
