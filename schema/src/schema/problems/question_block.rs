use crate::{
    ElementalQuestion, OrderFormat, OrderType, Paragraph,
    schema::renderer::{Html, Latex, Markdown, Problem, Renderer, Universal},
};

#[derive(Debug, Clone)]
pub struct QuestionSeries {
    pub content: Paragraph,

    pub questions: Vec<ElementalQuestion>,

    pub order_type: OrderType,
    pub order_format: OrderFormat,

    pub order_resume: bool,
}

// ── LaTeX ─────────────────────────────────────────────────────────────────────

impl Renderer<Latex, Problem> for QuestionSeries {
    /// Render the series for a *problem* sheet in LaTeX.
    ///
    /// 1. Lead-in content paragraph.
    /// 2. `\begin{enumerate}[label=…]` (via `enumitem`) where the label is
    ///    derived from [`OrderType`] and [`OrderFormat`], mirroring the [`List`]
    ///    renderer.  Each item delegates to [`ElementalQuestion`]'s LaTeX
    ///    `Problem` renderer.
    fn render(&self) -> anyhow::Result<String> {
        let mut out = String::new();

        out.push_str(&<Paragraph as Renderer<Latex, Universal>>::render(
            &self.content,
        )?);

        let env = if self.order_type == OrderType::Unordered {
            "\\begin{itemize}\n".to_owned()
        } else {
            let counter = self.order_type.latex_counter().unwrap();
            let label = self.order_format.wrap_latex(&format!("{}*", counter));
            let resume = if self.order_resume { ", resume" } else { "" };
            format!("\\begin{{enumerate}}[label={}{}]\n", label, resume)
        };
        out.push_str(&env);

        for question in &self.questions {
            let body = <ElementalQuestion as Renderer<Latex, Problem>>::render(question)?;
            out.push_str(&format!("  \\item {}\n", body.trim()));
        }

        if self.order_type == OrderType::Unordered {
            out.push_str("\\end{itemize}");
        } else {
            out.push_str("\\end{enumerate}");
        }

        Ok(out)
    }
}

// ── HTML ──────────────────────────────────────────────────────────────────────

impl Renderer<Html, Problem> for QuestionSeries {
    /// Render the series for a *problem* sheet as HTML.
    ///
    /// Wraps everything in `<div class="question-series">`.  Sub-questions are
    /// placed inside a `<ul>` or `<ol type="…">` matching the series'
    /// [`OrderType`] / [`OrderFormat`], mirroring the [`List`] HTML renderer.
    fn render(&self) -> anyhow::Result<String> {
        let mut out = String::new();

        out.push_str("<div class=\"question-series\">\n");
        out.push_str(&<Paragraph as Renderer<Html, Universal>>::render(
            &self.content,
        )?);
        out.push('\n');

        if self.order_type == OrderType::Unordered {
            out.push_str("<ul>\n");
            for question in &self.questions {
                let body = <ElementalQuestion as Renderer<Html, Problem>>::render(question)?;
                out.push_str(&format!("  <li>{}\n  </li>\n", body.trim()));
            }
            out.push_str("</ul>\n");
        } else {
            let type_attr = self.order_type.html_type().unwrap();
            let style_attr = match self.order_format {
                OrderFormat::Period => String::new(),
                OrderFormat::RightParenthesis => " style=\"list-style-type: '\\29 '\"".to_owned(),
                OrderFormat::Parenthesis => {
                    let css_type = match type_attr {
                        "1" => "decimal",
                        "a" => "lower-alpha",
                        "A" => "upper-alpha",
                        "i" => "lower-roman",
                        "I" => "upper-roman",
                        other => other,
                    };
                    format!(
                        " style=\"list-style-type: '(' counter(list-item, {}) ')'\"",
                        css_type
                    )
                }
                OrderFormat::None => " style=\"list-style-type:none;padding-left:0\"".to_owned(),
            };
            let resume_class = if self.order_resume { " class=\"resume\"" } else { "" };
            out.push_str(&format!("<ol type=\"{}\"{}{}>\n", type_attr, style_attr, resume_class));
            for question in &self.questions {
                let body = <ElementalQuestion as Renderer<Html, Problem>>::render(question)?;
                out.push_str(&format!("  <li>{}\n  </li>\n", body.trim()));
            }
            out.push_str("</ol>\n");
        }

        out.push_str("</div>");
        Ok(out)
    }
}

// ── Markdown ──────────────────────────────────────────────────────────────────

impl Renderer<Markdown, Problem> for QuestionSeries {
    /// Render the series for a *problem* sheet as Markdown.
    ///
    /// Mirrors the [`List`] Markdown renderer's strategy:
    /// - `Unordered` → `- item` bullets.
    /// - `Decimal + Period` → `1. item` (GFM numbered list).
    /// - All other type/format combinations → HTML `<ol>` fallback, since
    ///   CommonMark has no syntax for alphabetic or roman-numeral lists.
    fn render(&self) -> anyhow::Result<String> {
        let mut out = String::new();

        out.push_str(&<Paragraph as Renderer<Markdown, Universal>>::render(
            &self.content,
        )?);
        out.push('\n');

        match (&self.order_type, &self.order_format) {
            (OrderType::Unordered, _) => {
                for question in &self.questions {
                    let body =
                        <ElementalQuestion as Renderer<Markdown, Problem>>::render(question)?;
                    out.push_str(&format!("- {}\n", body.trim()));
                }
            }
            (OrderType::Decimal, OrderFormat::Period) => {
                for question in &self.questions {
                    let body =
                        <ElementalQuestion as Renderer<Markdown, Problem>>::render(question)?;
                    out.push_str(&format!("1. {}\n", body.trim()));
                }
            }
            _ => {
                // Fall back to HTML
                let type_attr = self.order_type.html_type().unwrap_or("1");
                let resume_class = if self.order_resume { " class=\"resume\"" } else { "" };
                out.push_str(&format!("<ol type=\"{}\"{}>\n", type_attr, resume_class));
                for question in &self.questions {
                    let body =
                        <ElementalQuestion as Renderer<Markdown, Problem>>::render(question)?;
                    out.push_str(&format!("  <li>{}\n  </li>\n", body.trim()));
                }
                out.push_str("</ol>\n");
            }
        }

        Ok(out)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Element, Text, schema::renderer::{Html, Latex, Markdown, Problem, Renderer},
    };

    fn para(s: &str) -> Paragraph {
        Paragraph::new(vec![Element::Text(s.parse::<Text>().unwrap())])
    }

    fn series(order_type: OrderType, order_format: OrderFormat) -> QuestionSeries {
        QuestionSeries {
            content: para("Answer all sub-questions."),
            questions: vec![
                ElementalQuestion {
                    id: "q1a".to_owned(),
                    content: para("What is photosynthesis?"),
                    answer: None,
                    solution: None,
                    choice_pool: None,
                    block_type: crate::QuestionBlock::None,
                },
                ElementalQuestion {
                    id: "q1b".to_owned(),
                    content: para("Name two reactants."),
                    answer: None,
                    solution: None,
                    choice_pool: None,
                    block_type: crate::QuestionBlock::Essay { lines: 2 },
                },
            ],
            order_type,
            order_format,
            order_resume: false,
        }
    }

    // ── LaTeX ────────────────────────────────────────────────────────────────

    #[test]
    fn latex_decimal_period() {
        let s_str =
            <QuestionSeries as Renderer<Latex, Problem>>::render(&series(
                OrderType::Decimal,
                OrderFormat::Period,
            ))
            .unwrap();
        assert!(s_str.contains("\\begin{enumerate}[label=\\arabic*.]"));
        assert!(s_str.contains("\\end{enumerate}"));
        assert!(s_str.contains("photosynthesis"));
        assert!(s_str.contains("\\rule{\\linewidth}"));
    }

    #[test]
    fn latex_alpha_parenthesis() {
        let s_str =
            <QuestionSeries as Renderer<Latex, Problem>>::render(&series(
                OrderType::LowercaseAlphabetic,
                OrderFormat::Parenthesis,
            ))
            .unwrap();
        assert!(s_str.contains("[label=(\\alph*)]"));
    }

    #[test]
    fn latex_unordered() {
        let s_str =
            <QuestionSeries as Renderer<Latex, Problem>>::render(&series(
                OrderType::Unordered,
                OrderFormat::None,
            ))
            .unwrap();
        assert!(s_str.contains("\\begin{itemize}"));
        assert!(s_str.contains("\\end{itemize}"));
    }

    // ── HTML ────────────────────────────────────────────────────────────────

    #[test]
    fn html_wraps_in_series_div() {
        let s_str =
            <QuestionSeries as Renderer<Html, Problem>>::render(&series(
                OrderType::Decimal,
                OrderFormat::Period,
            ))
            .unwrap();
        assert!(s_str.contains("<div class=\"question-series\">"));
        assert!(s_str.ends_with("</div>"));
    }

    #[test]
    fn html_ordered_list() {
        let s_str =
            <QuestionSeries as Renderer<Html, Problem>>::render(&series(
                OrderType::Decimal,
                OrderFormat::Period,
            ))
            .unwrap();
        assert!(s_str.contains("<ol type=\"1\">"));
        assert!(s_str.contains("Name two reactants"));
    }

    #[test]
    fn html_unordered_list() {
        let s_str =
            <QuestionSeries as Renderer<Html, Problem>>::render(&series(
                OrderType::Unordered,
                OrderFormat::None,
            ))
            .unwrap();
        assert!(s_str.contains("<ul>"));
    }

    // ── Markdown ─────────────────────────────────────────────────────────────

    #[test]
    fn markdown_decimal_list() {
        let s_str =
            <QuestionSeries as Renderer<Markdown, Problem>>::render(&series(
                OrderType::Decimal,
                OrderFormat::Period,
            ))
            .unwrap();
        assert!(s_str.contains("1. "));
        assert!(s_str.contains("photosynthesis"));
    }

    #[test]
    fn markdown_unordered_list() {
        let s_str =
            <QuestionSeries as Renderer<Markdown, Problem>>::render(&series(
                OrderType::Unordered,
                OrderFormat::None,
            ))
            .unwrap();
        assert!(s_str.contains("- "));
    }

    #[test]
    fn markdown_alpha_falls_back_to_html() {
        let s_str =
            <QuestionSeries as Renderer<Markdown, Problem>>::render(&series(
                OrderType::UppercaseAlphabetic,
                OrderFormat::Period,
            ))
            .unwrap();
        assert!(s_str.contains("<ol type=\"A\">"));
    }

    // ── resume ────────────────────────────────────────────────────────────────

    fn series_resume(order_type: OrderType, order_format: OrderFormat) -> QuestionSeries {
        let mut s = series(order_type, order_format);
        s.order_resume = true;
        s
    }

    #[test]
    fn latex_resume_adds_option() {
        let s_str =
            <QuestionSeries as Renderer<Latex, Problem>>::render(&series_resume(
                OrderType::Decimal,
                OrderFormat::Period,
            ))
            .unwrap();
        assert!(s_str.contains("[label=\\arabic*., resume]"));
    }

    #[test]
    fn latex_resume_unordered_unchanged() {
        // resume has no effect on itemize
        let with_resume =
            <QuestionSeries as Renderer<Latex, Problem>>::render(&series_resume(
                OrderType::Unordered,
                OrderFormat::None,
            ))
            .unwrap();
        let without_resume =
            <QuestionSeries as Renderer<Latex, Problem>>::render(&series(
                OrderType::Unordered,
                OrderFormat::None,
            ))
            .unwrap();
        assert_eq!(with_resume, without_resume);
    }

    #[test]
    fn html_resume_adds_class() {
        let s_str =
            <QuestionSeries as Renderer<Html, Problem>>::render(&series_resume(
                OrderType::Decimal,
                OrderFormat::Period,
            ))
            .unwrap();
        assert!(s_str.contains("class=\"resume\""));
    }

    #[test]
    fn markdown_resume_fallback_adds_class() {
        let s_str =
            <QuestionSeries as Renderer<Markdown, Problem>>::render(&series_resume(
                OrderType::UppercaseAlphabetic,
                OrderFormat::Period,
            ))
            .unwrap();
        assert!(s_str.contains("class=\"resume\""));
    }
}
