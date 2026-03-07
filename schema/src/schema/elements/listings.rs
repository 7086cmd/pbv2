use crate::schema::renderer::{Html, Latex, Markdown, Problem, Renderer, Solution};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProgrammingLanguage {
    Python,
    JavaScript,
    Rust,
    Cpp,
    Java,
    Go,
    Ruby,
    Swift,
    TypeScript,
}

impl ProgrammingLanguage {
    /// Returns the language identifier recognized by the LaTeX `listings` package.
    pub fn latex_language(&self) -> &'static str {
        match self {
            ProgrammingLanguage::Python => "Python",
            ProgrammingLanguage::JavaScript => "JavaScript",
            ProgrammingLanguage::Rust => "Rust",
            ProgrammingLanguage::Cpp => "C++",
            ProgrammingLanguage::Java => "Java",
            ProgrammingLanguage::Go => "Go",
            ProgrammingLanguage::Ruby => "Ruby",
            ProgrammingLanguage::Swift => "Swift",
            ProgrammingLanguage::TypeScript => "TypeScript",
        }
    }

    /// Returns the language identifier for Markdown fenced code blocks and HTML
    /// (e.g., used by Prism.js or highlight.js).
    pub fn code_fence(&self) -> &'static str {
        match self {
            ProgrammingLanguage::Python => "python",
            ProgrammingLanguage::JavaScript => "javascript",
            ProgrammingLanguage::Rust => "rust",
            ProgrammingLanguage::Cpp => "cpp",
            ProgrammingLanguage::Java => "java",
            ProgrammingLanguage::Go => "go",
            ProgrammingLanguage::Ruby => "ruby",
            ProgrammingLanguage::Swift => "swift",
            ProgrammingLanguage::TypeScript => "typescript",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CodeListing {
    pub language: ProgrammingLanguage,
    pub code: String,

    /// For blanks in code listings, we only allow strings. The tuple is (line number, column number, answer).
    pub blanks: Vec<(usize, usize, String)>,
}

impl CodeListing {
    /// Returns the code with each blank's answer replaced by underscores of the
    /// same character count, suitable for *problem* rendering.
    fn code_with_placeholder_blanks(&self) -> String {
        let mut lines: Vec<String> = self.code.lines().map(str::to_owned).collect();
        for (line_no, col_no, answer) in &self.blanks {
            let line_idx = line_no.saturating_sub(1);
            let col_idx = col_no.saturating_sub(1);
            if let Some(line) = lines.get_mut(line_idx) {
                let char_count = answer.chars().count();
                let char_indices: Vec<(usize, char)> = line.char_indices().collect();
                if col_idx < char_indices.len() {
                    let start_byte = char_indices[col_idx].0;
                    let end_byte = char_indices
                        .get(col_idx + char_count)
                        .map(|&(b, _)| b)
                        .unwrap_or(line.len());
                    let blank = "_".repeat(char_count);
                    line.replace_range(start_byte..end_byte, &blank);
                }
            }
        }
        lines.join("\n")
    }
}

/// Escapes `<`, `>`, and `&` so code is safe to embed in an HTML element.
fn html_escape_code(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

// ── LaTeX ─────────────────────────────────────────────────────────────────────

impl Renderer<Latex, Problem> for CodeListing {
    /// Render the code listing for a *problem* inside a `lstlisting` environment,
    /// with each blank's answer replaced by underscores of the same length.
    ///
    /// Relies on the `listings` package (already present in the project preamble)
    /// and the per-listing `language=` option to set syntax highlighting.
    fn render(&self) -> anyhow::Result<String> {
        let code = self.code_with_placeholder_blanks();
        Ok(format!(
            "\\begin{{lstlisting}}[language={}]\n{}\n\\end{{lstlisting}}",
            self.language.latex_language(),
            code
        ))
    }
}

impl Renderer<Latex, Solution> for CodeListing {
    /// Render the code listing for a *solution* inside a `lstlisting` environment,
    /// showing the full code with answers filled in.
    fn render(&self) -> anyhow::Result<String> {
        Ok(format!(
            "\\begin{{lstlisting}}[language={}]\n{}\n\\end{{lstlisting}}",
            self.language.latex_language(),
            self.code
        ))
    }
}

// ── HTML ──────────────────────────────────────────────────────────────────────

impl Renderer<Html, Problem> for CodeListing {
    /// Render the code listing as an HTML `<pre><code>` block for a *problem*,
    /// with blanks shown as underscores.  The `class="language-X"` attribute is
    /// compatible with Prism.js and highlight.js.
    fn render(&self) -> anyhow::Result<String> {
        let code = html_escape_code(&self.code_with_placeholder_blanks());
        Ok(format!(
            "<pre><code class=\"language-{}\">{}</code></pre>",
            self.language.code_fence(),
            code
        ))
    }
}

impl Renderer<Html, Solution> for CodeListing {
    /// Render the code listing as an HTML `<pre><code>` block for a *solution*,
    /// showing the full code with answers filled in.
    fn render(&self) -> anyhow::Result<String> {
        let code = html_escape_code(&self.code);
        Ok(format!(
            "<pre><code class=\"language-{}\">{}</code></pre>",
            self.language.code_fence(),
            code
        ))
    }
}

// ── Markdown ──────────────────────────────────────────────────────────────────

impl Renderer<Markdown, Problem> for CodeListing {
    /// Render the code listing as a Markdown fenced code block for a *problem*,
    /// with blanks shown as underscores.
    fn render(&self) -> anyhow::Result<String> {
        Ok(format!(
            "```{}\n{}\n```",
            self.language.code_fence(),
            self.code_with_placeholder_blanks()
        ))
    }
}

impl Renderer<Markdown, Solution> for CodeListing {
    /// Render the code listing as a Markdown fenced code block for a *solution*,
    /// showing the full code with answers filled in.
    fn render(&self) -> anyhow::Result<String> {
        Ok(format!(
            "```{}\n{}\n```",
            self.language.code_fence(),
            self.code
        ))
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::renderer::{Html, Latex, Markdown, Problem, Renderer, Solution};

    fn python_listing() -> CodeListing {
        CodeListing {
            language: ProgrammingLanguage::Python,
            code: "x = 42\nprint(x)".to_owned(),
            blanks: vec![],
        }
    }

    fn listing_with_blank() -> CodeListing {
        // Blank covers "42" at line 1, col 5 (1-based).
        CodeListing {
            language: ProgrammingLanguage::Rust,
            code: "let x = 42;".to_owned(),
            blanks: vec![(1, 9, "42".to_owned())],
        }
    }

    // ── ProgrammingLanguage helpers ───────────────────────────────────────────

    #[test]
    fn latex_language_names() {
        assert_eq!(ProgrammingLanguage::Python.latex_language(), "Python");
        assert_eq!(ProgrammingLanguage::Cpp.latex_language(), "C++");
        assert_eq!(ProgrammingLanguage::TypeScript.latex_language(), "TypeScript");
    }

    #[test]
    fn code_fence_names() {
        assert_eq!(ProgrammingLanguage::JavaScript.code_fence(), "javascript");
        assert_eq!(ProgrammingLanguage::Rust.code_fence(), "rust");
        assert_eq!(ProgrammingLanguage::Go.code_fence(), "go");
    }

    // ── LaTeX ─────────────────────────────────────────────────────────────────

    #[test]
    fn latex_problem_no_blanks() {
        let s = <CodeListing as Renderer<Latex, Problem>>::render(&python_listing()).unwrap();
        assert!(s.contains("\\begin{lstlisting}[language=Python]"));
        assert!(s.contains("x = 42"));
        assert!(s.contains("\\end{lstlisting}"));
    }

    #[test]
    fn latex_problem_blanks_replaced() {
        let s =
            <CodeListing as Renderer<Latex, Problem>>::render(&listing_with_blank()).unwrap();
        assert!(s.contains("__"), "expected underscores for the blank");
        assert!(!s.contains("42"), "answer should be hidden in problem mode");
    }

    #[test]
    fn latex_solution_shows_answer() {
        let s =
            <CodeListing as Renderer<Latex, Solution>>::render(&listing_with_blank()).unwrap();
        assert!(s.contains("42"), "answer must be visible in solution mode");
    }

    // ── HTML ──────────────────────────────────────────────────────────────────

    #[test]
    fn html_problem_structure() {
        let s = <CodeListing as Renderer<Html, Problem>>::render(&python_listing()).unwrap();
        assert!(s.starts_with("<pre><code class=\"language-python\">"));
        assert!(s.ends_with("</code></pre>"));
    }

    #[test]
    fn html_escapes_special_chars() {
        let listing = CodeListing {
            language: ProgrammingLanguage::Cpp,
            code: "if (a < b && c > d) {}".to_owned(),
            blanks: vec![],
        };
        let s = <CodeListing as Renderer<Html, Problem>>::render(&listing).unwrap();
        assert!(s.contains("&lt;"));
        assert!(s.contains("&gt;"));
        assert!(s.contains("&amp;"));
    }

    #[test]
    fn html_problem_blanks_replaced() {
        let s =
            <CodeListing as Renderer<Html, Problem>>::render(&listing_with_blank()).unwrap();
        assert!(s.contains("__"));
        assert!(!s.contains("42"));
    }

    #[test]
    fn html_solution_shows_answer() {
        let s =
            <CodeListing as Renderer<Html, Solution>>::render(&listing_with_blank()).unwrap();
        assert!(s.contains("42"));
    }

    // ── Markdown ──────────────────────────────────────────────────────────────

    #[test]
    fn markdown_problem_fenced_block() {
        let s = <CodeListing as Renderer<Markdown, Problem>>::render(&python_listing()).unwrap();
        assert!(s.starts_with("```python\n"));
        assert!(s.ends_with("\n```"));
    }

    #[test]
    fn markdown_problem_blanks_replaced() {
        let s =
            <CodeListing as Renderer<Markdown, Problem>>::render(&listing_with_blank()).unwrap();
        assert!(s.contains("__"));
        assert!(!s.contains("42"));
    }

    #[test]
    fn markdown_solution_shows_answer() {
        let s =
            <CodeListing as Renderer<Markdown, Solution>>::render(&listing_with_blank()).unwrap();
        assert!(s.contains("42"));
    }
}
