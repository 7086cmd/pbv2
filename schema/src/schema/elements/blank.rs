use anyhow::Result;

use crate::schema::{elements::Text, renderer::{Html, Latex, Markdown, Problem, Renderer, Solution}};

pub struct Blank {
    pub id: i32,
    /// The mark of the blank indicates the max mark that the student can receive for this blank. It is used for grading purposes and can be a floating-point number to allow for partial marks.
    pub mark: f32,
    /// We only allow text answers for blanks for now, but in the future we can support graphical answers as well.
    pub answer: Text,
    /// Written in `rem` units. In LaTeX, it is `\hspace*{`
    pub width: f32
}

impl Blank {
    pub fn new(id: i32, mark: f32, answer: Text, width: f32) -> Self {
        Self {
            id,
            mark,
            answer,
            width,
        }
    }

    pub fn from_answer(id: i32, mark: f32, answer: String) -> Result<Self> {
        let text = Text::parse(answer.as_str())?;
        Ok(Self::new(id, mark, text, answer.len() as f32 * 0.6)) // Estimate width based on character count, assuming an average character width of 0.6rem
    }
}

impl Renderer<Latex, Problem> for Blank {
    fn render(&self) -> Result<String> {
        Ok(format!("\\hspace*{{{}rem}}", self.width))
    }
}

impl Renderer<Markdown, Problem> for Blank {
    fn render(&self) -> Result<String> {
        Ok(format!("`{}`", "_".repeat(self.answer.content.len()))) // Render as a code block with underscores to indicate the blank
    }
}

impl Renderer<Html, Problem> for Blank {
    fn render(&self) -> Result<String> {
        Ok(format!("<span style=\"display:inline-block; width:{}rem; border-bottom:1px solid black;\"></span>", self.width))
    }
}

impl Renderer<Latex, Solution> for Blank {
    fn render(&self) -> Result<String> {
        Ok(format!("\\uline{{{}}}", <Text as Renderer<Latex, Solution>>::render(&self.answer)?)) // Render the answer underlined
    }
}

impl Renderer<Html, Solution> for Blank {
    fn render(&self) -> Result<String> {
        Ok(format!("<u>{}</u>", <Text as Renderer<Html, Solution>>::render(&self.answer)?)) // Render the answer as a code block
    }
}

impl Renderer<Markdown, Solution> for Blank {
    fn render(&self) -> Result<String> {
        Ok(format!("<u>{}</u>", <Text as Renderer<Markdown, Solution>>::render(&self.answer)?)) // Render the answer as a code block
    }
}
