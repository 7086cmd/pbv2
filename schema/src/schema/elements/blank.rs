use anyhow::Result;

use crate::{
    OrderType,
    schema::{
        elements::Text,
        renderer::{Html, Latex, Markdown, Problem, Renderer, Solution},
    },
};

#[derive(Clone, Debug)]
pub enum BlankAnswer {
    Text(Text),
    SingleChoice(usize, OrderType),
    MultipleChoice(Vec<usize>, OrderType),
}

impl Renderer<Latex, Solution> for BlankAnswer {
    fn render(&self) -> Result<String> {
        match self {
            BlankAnswer::Text(text) => <Text as Renderer<Latex, Problem>>::render(text),
            BlankAnswer::SingleChoice(index, order_type) => Ok(order_type.process(*index + 1)),
            BlankAnswer::MultipleChoice(indices, order_type) => Ok(indices
                .iter()
                .map(|i| order_type.process(*i + 1))
                .collect::<Vec<_>>()
                .join("")),
        }
    }
}

impl Renderer<Markdown, Solution> for BlankAnswer {
    fn render(&self) -> Result<String> {
        match self {
            BlankAnswer::Text(text) => <Text as Renderer<Markdown, Problem>>::render(text),
            BlankAnswer::SingleChoice(index, order_type) => Ok(order_type.process(*index + 1)),
            BlankAnswer::MultipleChoice(indices, order_type) => Ok(indices
                .iter()
                .map(|i| order_type.process(*i + 1))
                .collect::<Vec<_>>()
                .join("")),
        }
    }
}

impl Renderer<Html, Solution> for BlankAnswer {
    fn render(&self) -> Result<String> {
        match self {
            BlankAnswer::Text(text) => <Text as Renderer<Html, Problem>>::render(text),
            BlankAnswer::SingleChoice(index, order_type) => Ok(order_type.process(*index + 1)),
            BlankAnswer::MultipleChoice(indices, order_type) => Ok(indices
                .iter()
                .map(|i| order_type.process(*i + 1))
                .collect::<Vec<_>>()
                .join("")),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Blank {
    pub id: i32,
    /// The mark of the blank indicates the max mark that the student can receive for this blank. It is used for grading purposes and can be a floating-point number to allow for partial marks.
    pub mark: f32,
    /// We only allow text answers for blanks for now, but in the future we can support graphical answers as well.
    pub answer: BlankAnswer,
    /// Written in `rem` units. In LaTeX, it is `\hspace*{`
    pub width: f32,
}

impl Blank {
    pub fn new(id: i32, mark: f32, answer: BlankAnswer, width: f32) -> Self {
        Self {
            id,
            mark,
            answer,
            width,
        }
    }

    pub fn from_answer(id: i32, mark: f32, answer: String) -> Result<Self> {
        let text = Text::parse(answer.as_str())?;
        Ok(Self::new(
            id,
            mark,
            BlankAnswer::Text(text),
            answer.len() as f32 * 0.6,
        )) // Estimate width based on character count, assuming an average character width of 0.6rem
    }
}

impl Renderer<Latex, Problem> for Blank {
    fn render(&self) -> Result<String> {
        Ok(format!("\\hspace*{{{}rem}}", self.width))
    }
}

impl Renderer<Markdown, Problem> for Blank {
    fn render(&self) -> Result<String> {
        Ok(format!("`{}`", "_".repeat((self.width * 1.2) as usize))) // Render as a code block with underscores to indicate the blank
    }
}

impl Renderer<Html, Problem> for Blank {
    fn render(&self) -> Result<String> {
        Ok(format!(
            "<span style=\"display:inline-block; width:{}rem; border-bottom:1px solid black;\"></span>",
            self.width
        ))
    }
}

impl Renderer<Latex, Solution> for Blank {
    fn render(&self) -> Result<String> {
        Ok(format!(
            "\\uline{{{}}}",
            <BlankAnswer as Renderer<Latex, Solution>>::render(&self.answer)?
        )) // Render the answer underlined
    }
}

impl Renderer<Html, Solution> for Blank {
    fn render(&self) -> Result<String> {
        Ok(format!(
            "<u>{}</u>",
            <BlankAnswer as Renderer<Html, Solution>>::render(&self.answer)?
        )) // Render the answer as a code block
    }
}

impl Renderer<Markdown, Solution> for Blank {
    fn render(&self) -> Result<String> {
        Ok(format!(
            "<u>{}</u>",
            <BlankAnswer as Renderer<Markdown, Solution>>::render(&self.answer)?
        )) // Render the answer as a code block
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        OrderType,
        schema::renderer::{Html, Latex, Markdown, Problem, Renderer, Solution},
    };

    // ── Constructors ──────────────────────────────────────────────────────────

    #[test]
    fn new_stores_all_fields() {
        let answer = BlankAnswer::Text("x".parse().unwrap());
        let b = Blank::new(7, 2.5, answer, 4.0);
        assert_eq!(b.id, 7);
        assert_eq!(b.mark, 2.5);
        assert_eq!(b.width, 4.0);
    }

    #[test]
    fn from_answer_parses_text() {
        let b = Blank::from_answer(1, 1.0, "hello".to_string()).unwrap();
        assert_eq!(b.id, 1);
        assert_eq!(b.mark, 1.0);
        // width ≈ len * 0.6 = 5 * 0.6 = 3.0
        assert!((b.width - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn from_answer_width_scales_with_length() {
        let short = Blank::from_answer(1, 1.0, "ab".to_string()).unwrap();
        let long = Blank::from_answer(2, 1.0, "abcdefghij".to_string()).unwrap();
        assert!(long.width > short.width);
    }

    // ── Problem renderer – LaTeX ──────────────────────────────────────────────

    #[test]
    fn latex_problem_emits_hspace() {
        let b = Blank::new(1, 1.0, BlankAnswer::Text("x".parse().unwrap()), 5.0);
        let out = <Blank as Renderer<Latex, Problem>>::render(&b).unwrap();
        assert_eq!(out, "\\hspace*{5rem}");
    }

    #[test]
    fn latex_problem_hspace_uses_width_field() {
        let b = Blank::new(1, 1.0, BlankAnswer::Text("x".parse().unwrap()), 12.5);
        let out = <Blank as Renderer<Latex, Problem>>::render(&b).unwrap();
        assert!(out.contains("12.5rem"));
    }

    // ── Problem renderer – HTML ───────────────────────────────────────────────

    #[test]
    fn html_problem_emits_span_with_border() {
        let b = Blank::new(1, 1.0, BlankAnswer::Text("x".parse().unwrap()), 6.0);
        let out = <Blank as Renderer<Html, Problem>>::render(&b).unwrap();
        assert!(out.starts_with("<span"));
        assert!(out.contains("border-bottom"));
        assert!(out.contains("6rem"));
    }

    #[test]
    fn html_problem_contains_no_answer_text() {
        let b = Blank::from_answer(1, 1.0, "secret".to_string()).unwrap();
        let out = <Blank as Renderer<Html, Problem>>::render(&b).unwrap();
        assert!(!out.contains("secret"));
    }

    // ── Problem renderer – Markdown ───────────────────────────────────────────

    #[test]
    fn md_problem_emits_code_span_of_underscores() {
        let b = Blank::new(1, 1.0, BlankAnswer::Text("x".parse().unwrap()), 5.0);
        let out = <Blank as Renderer<Markdown, Problem>>::render(&b).unwrap();
        // width 5.0 → 5.0 * 1.2 = 6 underscores
        assert_eq!(out, "`______`");
    }

    #[test]
    fn md_problem_underscore_count_matches_width() {
        let b = Blank::new(1, 1.0, BlankAnswer::Text("x".parse().unwrap()), 10.0);
        let out = <Blank as Renderer<Markdown, Problem>>::render(&b).unwrap();
        // width 10.0 → 10.0 * 1.2 = 12 underscores
        let inner = out.trim_matches('`');
        assert_eq!(inner.len(), 12);
        assert!(inner.chars().all(|c| c == '_'));
    }

    #[test]
    fn md_problem_contains_no_answer_text() {
        let b = Blank::from_answer(1, 1.0, "hidden".to_string()).unwrap();
        let out = <Blank as Renderer<Markdown, Problem>>::render(&b).unwrap();
        assert!(!out.contains("hidden"));
    }

    // ── Solution renderer – Text answer ──────────────────────────────────────

    #[test]
    fn latex_solution_text_is_underlined() {
        let b = Blank::new(1, 1.0, BlankAnswer::Text("42".parse().unwrap()), 3.0);
        let out = <Blank as Renderer<Latex, Solution>>::render(&b).unwrap();
        assert!(out.starts_with("\\uline{"));
        assert!(out.contains("42"));
    }

    #[test]
    fn html_solution_text_is_wrapped_in_u_tag() {
        let b = Blank::new(1, 1.0, BlankAnswer::Text("42".parse().unwrap()), 3.0);
        let out = <Blank as Renderer<Html, Solution>>::render(&b).unwrap();
        assert!(out.starts_with("<u>"));
        assert!(out.ends_with("</u>"));
        assert!(out.contains("42"));
    }

    #[test]
    fn md_solution_text_is_wrapped_in_u_tag() {
        let b = Blank::new(1, 1.0, BlankAnswer::Text("42".parse().unwrap()), 3.0);
        let out = <Blank as Renderer<Markdown, Solution>>::render(&b).unwrap();
        assert!(out.starts_with("<u>"));
        assert!(out.ends_with("</u>"));
        assert!(out.contains("42"));
    }

    // ── Solution renderer – SingleChoice ─────────────────────────────────────

    #[test]
    fn latex_solution_single_choice_decimal() {
        // index 1 → process(2) → "2"
        let b = Blank::new(
            1,
            1.0,
            BlankAnswer::SingleChoice(1, OrderType::Decimal),
            3.0,
        );
        let out = <Blank as Renderer<Latex, Solution>>::render(&b).unwrap();
        assert_eq!(out, "\\uline{2}");
    }

    #[test]
    fn html_solution_single_choice_uppercase_alpha() {
        // index 0 → process(1) → "A"
        let b = Blank::new(
            1,
            1.0,
            BlankAnswer::SingleChoice(0, OrderType::UppercaseAlphabetic),
            3.0,
        );
        let out = <Blank as Renderer<Html, Solution>>::render(&b).unwrap();
        assert_eq!(out, "<u>A</u>");
    }

    #[test]
    fn md_solution_single_choice_lowercase_alpha() {
        // index 2 → process(3) → "c"
        let b = Blank::new(
            1,
            1.0,
            BlankAnswer::SingleChoice(2, OrderType::LowercaseAlphabetic),
            3.0,
        );
        let out = <Blank as Renderer<Markdown, Solution>>::render(&b).unwrap();
        assert_eq!(out, "<u>c</u>");
    }

    #[test]
    fn solution_single_choice_uppercase_roman() {
        // index 3 → process(4) → "IV"
        let b = Blank::new(
            1,
            1.0,
            BlankAnswer::SingleChoice(3, OrderType::UppercaseRoman),
            3.0,
        );
        let out = <Blank as Renderer<Latex, Solution>>::render(&b).unwrap();
        assert_eq!(out, "\\uline{IV}");
    }

    #[test]
    fn solution_single_choice_lowercase_roman() {
        // index 0 → process(1) → "i"
        let b = Blank::new(
            1,
            1.0,
            BlankAnswer::SingleChoice(0, OrderType::LowercaseRoman),
            3.0,
        );
        let out = <Blank as Renderer<Latex, Solution>>::render(&b).unwrap();
        assert_eq!(out, "\\uline{i}");
    }

    // ── Solution renderer – MultipleChoice ───────────────────────────────────

    #[test]
    fn latex_solution_multiple_choices_joined_by_comma() {
        // indices [0, 2] → process(1)="A", process(3)="C" → "AC"
        let b = Blank::new(
            1,
            1.0,
            BlankAnswer::MultipleChoice(vec![0, 2], OrderType::UppercaseAlphabetic),
            3.0,
        );
        let out = <Blank as Renderer<Latex, Solution>>::render(&b).unwrap();
        assert_eq!(out, "\\uline{AC}");
    }

    #[test]
    fn html_solution_multiple_choices_decimal() {
        // indices [0, 1, 3] → "124"
        let b = Blank::new(
            1,
            1.0,
            BlankAnswer::MultipleChoice(vec![0, 1, 3], OrderType::Decimal),
            3.0,
        );
        let out = <Blank as Renderer<Html, Solution>>::render(&b).unwrap();
        assert_eq!(out, "<u>124</u>");
    }

    #[test]
    fn md_solution_multiple_choices_lowercase_alpha() {
        // indices [0, 3, 5] → "adf"
        let b = Blank::new(
            1,
            1.0,
            BlankAnswer::MultipleChoice(vec![0, 3, 5], OrderType::LowercaseAlphabetic),
            3.0,
        );
        let out = <Blank as Renderer<Markdown, Solution>>::render(&b).unwrap();
        assert_eq!(out, "<u>adf</u>");
    }

    #[test]
    fn solution_multiple_choices_single_element() {
        // Degenerate: only one index — still formats correctly.
        let b = Blank::new(
            1,
            1.0,
            BlankAnswer::MultipleChoice(vec![1], OrderType::UppercaseAlphabetic),
            3.0,
        );
        let out = <Blank as Renderer<Latex, Solution>>::render(&b).unwrap();
        assert_eq!(out, "\\uline{B}");
    }
}
