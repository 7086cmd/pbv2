use crate::{
    List, Paragraph,
    schema::renderer::{Html, Latex, Markdown, Problem, Renderer, Universal},
};

#[derive(Clone, Debug, PartialEq)]
pub enum QuestionBlock {
    /// Provides answer lines for students to write their answers on.
    /// The number of lines can be specified by the `lines` field.
    Essay { lines: usize },
    /// Provides a space for students to write their proof. The `space` field specifies the amount of space (`rems`) allocated for the proof.
    Proof { space: f32 },
    /// Provides a space for students to write their solution. The `space` field specifies the amount of space (`rems`) allocated for the solution.
    Solve { space: f32 },
    None,
}

#[derive(Clone, Debug)]
pub struct ElementalQuestion {
    pub id: String,
    pub content: Paragraph,
    pub answer: Option<Paragraph>,
    pub solution: Option<Paragraph>,

    pub choice_pool: Option<List>,
    pub block_type: QuestionBlock,
}

// ── LaTeX ─────────────────────────────────────────────────────────────────────

impl Renderer<Latex, Problem> for ElementalQuestion {
    /// Render the question for a *problem* (student-facing) sheet in LaTeX.
    ///
    /// Layout:
    /// 1. Question content paragraph.
    /// 2. Multiple-choice pool (if any) – rendered as an `enumerate` list, one
    ///    choice per item, using the pool's own [`OrderType`] / [`OrderFormat`].
    /// 3. Answer block determined by [`QuestionBlock`]:
    ///    - `Essay`  → `lines` ruled lines spaced 1.8 em apart.
    ///    - `Proof`  → a plain `\vspace` of the given size for freehand work.
    ///    - `Solve`  → same as `Proof` but labelled _Solution:_.
    ///    - `None`   → nothing.
    fn render(&self) -> anyhow::Result<String> {
        let mut out = String::new();

        // 1. Content
        out.push_str(&<Paragraph as Renderer<Latex, Universal>>::render(
            &self.content,
        )?);

        // 2. Choice pool
        if let Some(choices) = &self.choice_pool {
            out.push_str(&<List as Renderer<Latex, Universal>>::render(choices)?);
            out.push('\n');
        }

        // 3. Answer block
        match &self.block_type {
            QuestionBlock::Essay { lines } => {
                out.push('\n');
                for _ in 0..*lines {
                    // A horizontal rule 15 cm wide, spaced by 1.8em
                    out.push_str("\\noindent\\rule{\\linewidth}{0.4pt}\\vspace{1.4em}\n\n");
                }
            }
            QuestionBlock::Proof { space } => {
                out.push_str(&format!("\n\\vspace{{{:.2}em}}\n", space));
            }
            QuestionBlock::Solve { space } => {
                out.push_str(&format!(
                    "\n\\noindent\\vspace{{{:.2}em}}\n",
                    space
                ));
            }
            QuestionBlock::None => {}
        }

        Ok(out)
    }
}

// ── HTML ──────────────────────────────────────────────────────────────────────

impl Renderer<Html, Problem> for ElementalQuestion {
    /// Render the question for a *problem* sheet as HTML.
    ///
    /// Wraps everything in `<div class="question" data-id="…">`.
    /// - Choice pool → `<ol class="choices">` with `list-style-type` matching
    ///   the pool's [`OrderType`].
    /// - `Essay`  → `<div class="answer-lines">` containing `lines` ruled spans.
    /// - `Proof`  → blank `<div>` with an explicit pixel height.
    /// - `Solve`  → same with an _"Solution:"_ label.
    fn render(&self) -> anyhow::Result<String> {
        let mut out = String::new();
        out.push_str(&format!(
            "<div class=\"question\" data-id=\"{}\">\n",
            self.id
        ));

        // Content
        out.push_str(&<Paragraph as Renderer<Html, Universal>>::render(
            &self.content,
        )?);

        // Choice pool
        if let Some(choices) = &self.choice_pool {
            out.push_str(&<List as Renderer<Html, Universal>>::render(choices)?);
            out.push('\n');
        }

        // Answer block
        match &self.block_type {
            QuestionBlock::Essay { lines } => {
                out.push_str("<div class=\"answer-lines\">\n");
                let line_height_px = 36u32;
                for _ in 0..*lines {
                    out.push_str(&format!(
                        "  <div style=\"border-bottom:1px solid #000; height:{}px; margin-bottom:4px;\"></div>\n",
                        line_height_px
                    ));
                }
                out.push_str("</div>\n");
            }
            QuestionBlock::Proof { space } => {
                let px = (*space * 16.0) as u32; // 1 rem ≈ 16 px
                out.push_str(&format!(
                    "<div class=\"proof-space\" style=\"height:{}px;\"></div>\n",
                    px
                ));
            }
            QuestionBlock::Solve { space } => {
                let px = (*space * 16.0) as u32;
                out.push_str(&format!(
                    "<div class=\"solve-space\"><em>Solution:</em><div style=\"height:{}px;\"></div></div>\n",
                    px
                ));
            }
            QuestionBlock::None => {}
        }

        out.push_str("</div>");
        Ok(out)
    }
}

// ── Markdown ──────────────────────────────────────────────────────────────────

impl Renderer<Markdown, Problem> for ElementalQuestion {
    /// Render the question for a *problem* sheet as Markdown.
    ///
    /// - Choice pool → lettered list `(A)`, `(B)`, …
    /// - `Essay`  → `lines` blank lines represented as `___` separators.
    /// - `Proof` / `Solve` → an HTML `<div>` spacer (Markdown is HTML-aware).
    fn render(&self) -> anyhow::Result<String> {
        let mut out = String::new();

        // Content
        out.push_str(&<Paragraph as Renderer<Markdown, Universal>>::render(
            &self.content,
        )?);
        out.push('\n');

        // Choice pool
        if let Some(choices) = &self.choice_pool {
            out.push_str(&<List as Renderer<Markdown, Universal>>::render(choices)?);
            out.push('\n');
        }

        // Answer block
        match &self.block_type {
            QuestionBlock::Essay { lines } => {
                out.push('\n');
                for _ in 0..*lines {
                    out.push_str("___\n\n");
                }
            }
            QuestionBlock::Proof { space } => {
                let px = (*space * 16.0) as u32;
                out.push_str(&format!(
                    "\n<div class=\"proof-space\" style=\"height:{}px;\"></div>\n",
                    px
                ));
            }
            QuestionBlock::Solve { space } => {
                let px = (*space * 16.0) as u32;
                out.push_str(&format!(
                    "\n**Solution:**\n<div style=\"height:{}px;\"></div>\n",
                    px
                ));
            }
            QuestionBlock::None => {}
        }

        Ok(out)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Element, List, OrderFormat, OrderType, Text,
        schema::renderer::{Html, Latex, Markdown, Problem, Renderer},
    };

    fn para(s: &str) -> Paragraph {
        Paragraph::new(vec![Element::Text(s.parse::<Text>().unwrap())])
    }

    fn base_question(block: QuestionBlock) -> ElementalQuestion {
        ElementalQuestion {
            id: "q1".to_owned(),
            content: para("Describe the water cycle."),
            answer: None,
            solution: None,
            choice_pool: None,
            block_type: block,
        }
    }

    fn mcq_question() -> ElementalQuestion {
        ElementalQuestion {
            id: "q2".to_owned(),
            content: para("Which gas makes up most of the atmosphere?"),
            answer: None,
            solution: None,
            choice_pool: Some(List {
                items: vec![
                    para("Oxygen"),
                    para("Nitrogen"),
                    para("Carbon dioxide"),
                    para("Argon"),
                ],
                order_type: OrderType::UppercaseAlphabetic,
                order_format: OrderFormat::Parenthesis,
            }),
            block_type: QuestionBlock::None,
        }
    }

    // ── LaTeX ─────────────────────────────────────────────────────────────────

    #[test]
    fn latex_essay_lines() {
        let q = base_question(QuestionBlock::Essay { lines: 3 });
        let s = <ElementalQuestion as Renderer<Latex, Problem>>::render(&q).unwrap();
        assert!(s.contains("\\rule{\\linewidth}"), "should have ruled lines");
        assert_eq!(s.matches("\\rule{\\linewidth}").count(), 3);
    }

    #[test]
    fn latex_proof_space() {
        let q = base_question(QuestionBlock::Proof { space: 10.0 });
        let s = <ElementalQuestion as Renderer<Latex, Problem>>::render(&q).unwrap();
        assert!(s.contains("\\vspace{10.00em}"));
    }

    #[test]
    fn latex_solve_label() {
        let q = base_question(QuestionBlock::Solve { space: 8.0 });
        let s = <ElementalQuestion as Renderer<Latex, Problem>>::render(&q).unwrap();
        assert!(s.contains("\\vspace{8.00em}"));
    }

    #[test]
    fn latex_none_block() {
        let q = base_question(QuestionBlock::None);
        let s = <ElementalQuestion as Renderer<Latex, Problem>>::render(&q).unwrap();
        assert!(s.contains("water cycle"));
        assert!(!s.contains("\\vspace"));
    }

    #[test]
    fn latex_choice_pool() {
        let q = mcq_question();
        let s = <ElementalQuestion as Renderer<Latex, Problem>>::render(&q).unwrap();
        assert!(s.contains("\\begin{enumerate}"), "should contain enumerate env");
        assert!(s.contains("Nitrogen"));
        assert!(s.contains("Oxygen"));
    }

    // ── HTML ──────────────────────────────────────────────────────────────────

    #[test]
    fn html_wraps_in_question_div() {
        let q = base_question(QuestionBlock::None);
        let s = <ElementalQuestion as Renderer<Html, Problem>>::render(&q).unwrap();
        assert!(s.contains("<div class=\"question\" data-id=\"q1\">"));
        assert!(s.ends_with("</div>"));
    }

    #[test]
    fn html_essay_lines() {
        let q = base_question(QuestionBlock::Essay { lines: 2 });
        let s = <ElementalQuestion as Renderer<Html, Problem>>::render(&q).unwrap();
        assert!(s.contains("answer-lines"));
        assert_eq!(s.matches("border-bottom").count(), 2);
    }

    #[test]
    fn html_choice_pool() {
        let q = mcq_question();
        let s = <ElementalQuestion as Renderer<Html, Problem>>::render(&q).unwrap();
        assert!(s.contains("<ol"), "should contain an ordered list");
        assert!(s.contains("Oxygen"));
        assert!(s.contains("Nitrogen"));
    }

    // ── Markdown ──────────────────────────────────────────────────────────────

    #[test]
    fn markdown_essay_underscores() {
        let q = base_question(QuestionBlock::Essay { lines: 2 });
        let s = <ElementalQuestion as Renderer<Markdown, Problem>>::render(&q).unwrap();
        assert_eq!(s.matches("___").count(), 2);
    }

    #[test]
    fn markdown_choice_pool_lettered() {
        let q = mcq_question();
        let s = <ElementalQuestion as Renderer<Markdown, Problem>>::render(&q).unwrap();
        // UppercaseAlphabetic + Parenthesis has no native Markdown encoding,
        // so the List renderer falls back to HTML (<ol type="A">).
        assert!(s.contains("<ol"), "expected an ordered list element");
        assert!(s.contains("Nitrogen"));
        assert!(s.contains("Oxygen"));
    }

    #[test]
    fn markdown_solve_label() {
        let q = base_question(QuestionBlock::Solve { space: 5.0 });
        let s = <ElementalQuestion as Renderer<Markdown, Problem>>::render(&q).unwrap();
        assert!(s.contains("**Solution:**"));
    }
}
