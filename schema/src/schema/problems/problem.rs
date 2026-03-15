use crate::{
    ElementalQuestion, Paragraph,
    schema::{
        problems::ProblemCategory,
        renderer::{Html, Latex, Markdown, Problem, Renderer, Universal},
    },
};

#[derive(Debug, Clone)]
pub struct SingleProblem {
    pub problem: ElementalQuestion,
    pub category: ProblemCategory,
}

#[derive(Debug, Clone)]
pub struct ProblemGroup {
    pub material: Paragraph,
    pub problems: Vec<ElementalQuestion>,
    pub category: ProblemCategory,
}

// ── SingleProblem ─────────────────────────────────────────────────────────────
//
// `ProblemCategory` is metadata (DB references) and is not included in the
// rendered output.

impl Renderer<Latex, Problem> for SingleProblem {
    fn render(&self) -> anyhow::Result<String> {
        <ElementalQuestion as Renderer<Latex, Problem>>::render(&self.problem)
    }
}

impl Renderer<Html, Problem> for SingleProblem {
    fn render(&self) -> anyhow::Result<String> {
        <ElementalQuestion as Renderer<Html, Problem>>::render(&self.problem)
    }
}

impl Renderer<Markdown, Problem> for SingleProblem {
    fn render(&self) -> anyhow::Result<String> {
        <ElementalQuestion as Renderer<Markdown, Problem>>::render(&self.problem)
    }
}

// ── ProblemGroup ──────────────────────────────────────────────────────────────
//
// A `ProblemGroup` has shared reading material followed by a list of problems.
// In LaTeX the material is typeset as a `mdframed` box (already in the
// preamble) so it visually groups with its sub-problems.

impl Renderer<Latex, Problem> for ProblemGroup {
    /// Render the group for a *problem* sheet in LaTeX.
    ///
    /// Layout:
    /// 1. Material paragraph inside `\begin{mdframed} … \end{mdframed}`.
    /// 2. Each [`ElementalQuestion`] rendered in sequence.
    fn render(&self) -> anyhow::Result<String> {
        let mut out = String::new();

        let material = <Paragraph as Renderer<Latex, Universal>>::render(&self.material)?;
        out.push_str(&format!("\\begin{{mdframed}}\n{}\n\\end{{mdframed}}\n", material.trim()));

        for problem in &self.problems {
            out.push_str(&<ElementalQuestion as Renderer<Latex, Problem>>::render(problem)?);
            out.push('\n');
        }

        Ok(out)
    }
}

impl Renderer<Html, Problem> for ProblemGroup {
    /// Render the group for a *problem* sheet as HTML.
    ///
    /// Wraps everything in `<div class="problem-group">`.  The material is
    /// placed in a `<blockquote class="material">` to visually distinguish it
    /// from the sub-problems.
    fn render(&self) -> anyhow::Result<String> {
        let mut out = String::new();

        out.push_str("<div class=\"problem-group\">\n");

        let material = <Paragraph as Renderer<Html, Universal>>::render(&self.material)?;
        out.push_str(&format!("<blockquote class=\"material\">{}\n</blockquote>\n", material));

        for problem in &self.problems {
            out.push_str(&<ElementalQuestion as Renderer<Html, Problem>>::render(problem)?);
            out.push('\n');
        }

        out.push_str("</div>");
        Ok(out)
    }
}

impl Renderer<Markdown, Problem> for ProblemGroup {
    /// Render the group for a *problem* sheet as Markdown.
    ///
    /// The material is rendered as a Markdown blockquote (`> …`), followed by
    /// each sub-problem rendered in sequence.
    fn render(&self) -> anyhow::Result<String> {
        let mut out = String::new();

        let material = <Paragraph as Renderer<Markdown, Universal>>::render(&self.material)?;
        for line in material.trim().lines() {
            out.push_str(&format!("> {}\n", line));
        }
        out.push('\n');

        for problem in &self.problems {
            out.push_str(&<ElementalQuestion as Renderer<Markdown, Problem>>::render(problem)?);
            out.push('\n');
        }

        Ok(out)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Element, List, OrderFormat, OrderType, QuestionBlock, Text,
        schema::renderer::{Html, Latex, Markdown, Problem, Renderer},
    };

    fn para(s: &str) -> Paragraph {
        Paragraph::new(vec![Element::Text(s.parse::<Text>().unwrap())])
    }

    fn category() -> ProblemCategory {
        ProblemCategory {
            cirriculum: 1,
            subject: 2,
            grade: 10,
            categories: vec!["Biology".to_owned()],
            origin: None,
        }
    }

    fn simple_question(id: &str, text: &str) -> ElementalQuestion {
        ElementalQuestion {
            id: id.to_owned(),
            content: para(text),
            answer: None,
            solution: None,
            choice_pool: None,
            block_type: QuestionBlock::None,
        }
    }

    fn q_problem(id: &str, text: &str) -> ElementalQuestion {
        simple_question(id, text)
    }

    fn subquestion_problem() -> ElementalQuestion {
        ElementalQuestion {
            id: "q_sub".to_owned(),
            content: para("Answer all."),
            answer: None,
            solution: None,
            choice_pool: Some(List {
                items: vec![para("Part A"), para("Part B")],
                order_type: OrderType::LowercaseAlphabetic,
                order_format: OrderFormat::Parenthesis,
            }),
            block_type: QuestionBlock::None,
        }
    }

    // ── ElementalQuestion ──────────────────────────────────────────────────────

    #[test]
    fn elemental_question_latex() {
        let s = <ElementalQuestion as Renderer<Latex, Problem>>::render(&q_problem("q1", "Define osmosis.")).unwrap();
        assert!(s.contains("osmosis"));
    }

    #[test]
    fn elemental_subquestion_html() {
        let s = <ElementalQuestion as Renderer<Html, Problem>>::render(&subquestion_problem()).unwrap();
        assert!(s.contains("<ol"));
        assert!(s.contains("Part A"));
    }

    #[test]
    fn elemental_subquestion_markdown() {
        let s = <ElementalQuestion as Renderer<Markdown, Problem>>::render(&subquestion_problem()).unwrap();
        assert!(s.contains("Part B"));
    }

    // ── SingleProblem ──────────────────────────────────────────────────────────

    #[test]
    fn single_problem_latex() {
        let sp = SingleProblem {
            problem: q_problem("q1", "What is ATP?"),
            category: category(),
        };
        let s = <SingleProblem as Renderer<Latex, Problem>>::render(&sp).unwrap();
        assert!(s.contains("ATP"));
    }

    #[test]
    fn single_problem_html() {
        let sp = SingleProblem {
            problem: q_problem("q1", "What is ATP?"),
            category: category(),
        };
        let s = <SingleProblem as Renderer<Html, Problem>>::render(&sp).unwrap();
        assert!(s.contains("ATP"));
        assert!(s.contains("<div class=\"question\""));
    }

    #[test]
    fn single_problem_markdown() {
        let sp = SingleProblem {
            problem: q_problem("q1", "What is ATP?"),
            category: category(),
        };
        let s = <SingleProblem as Renderer<Markdown, Problem>>::render(&sp).unwrap();
        assert!(s.contains("ATP"));
    }

    // ── ProblemGroup ───────────────────────────────────────────────────────────

    #[test]
    fn group_latex_mdframed() {
        let pg = ProblemGroup {
            material: para("Read the passage."),
            problems: vec![q_problem("q1", "Q1"), q_problem("q2", "Q2")],
            category: category(),
        };
        let s = <ProblemGroup as Renderer<Latex, Problem>>::render(&pg).unwrap();
        assert!(s.contains("\\begin{mdframed}"));
        assert!(s.contains("\\end{mdframed}"));
        assert!(s.contains("passage"));
        assert!(s.contains("Q1"));
        assert!(s.contains("Q2"));
    }

    #[test]
    fn group_html_blockquote() {
        let pg = ProblemGroup {
            material: para("Read the passage."),
            problems: vec![q_problem("q1", "Q1")],
            category: category(),
        };
        let s = <ProblemGroup as Renderer<Html, Problem>>::render(&pg).unwrap();
        assert!(s.contains("<div class=\"problem-group\">"));
        assert!(s.contains("<blockquote class=\"material\">"));
        assert!(s.contains("passage"));
        assert!(s.ends_with("</div>"));
    }

    #[test]
    fn group_markdown_blockquote() {
        let pg = ProblemGroup {
            material: para("Read the passage."),
            problems: vec![q_problem("q1", "Q1")],
            category: category(),
        };
        let s = <ProblemGroup as Renderer<Markdown, Problem>>::render(&pg).unwrap();
        assert!(s.contains("> "), "material should be a blockquote");
        assert!(s.contains("passage"));
        assert!(s.contains("Q1"));
    }
}