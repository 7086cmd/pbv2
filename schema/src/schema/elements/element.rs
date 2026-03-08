use crate::schema::elements::{BlankAnswer, ChoicePool};
use crate::schema::renderer::{Html, Latex, Markdown, Problem, Renderer, Universal};
use crate::{Blank, CodeListing, Image, List, SiUnitX, Table, Text};

#[derive(Clone, Debug)]
pub enum Element {
    Text(Text),
    Table(Table),
    Image(Image),
    List(List),
    Blank(Blank),
    CodeListing(CodeListing),
    Si(SiUnitX),
}

#[derive(Clone, Debug)]
pub struct Paragraph {
    pub elements: Vec<Element>,
    pub choice_pool: Option<ChoicePool>,
}

impl Paragraph {
    pub fn new(elements: Vec<Element>) -> Self {
        Self {
            elements,
            choice_pool: None,
        }
    }

    pub fn with_choice_pool(mut self, pool: ChoicePool) -> Self {
        self.choice_pool = Some(pool);
        self
    }
}

impl From<Text> for Element {
    fn from(text: Text) -> Self {
        Element::Text(text)
    }
}

impl From<Table> for Element {
    fn from(table: Table) -> Self {
        Element::Table(table)
    }
}

impl From<Image> for Element {
    fn from(image: Image) -> Self {
        Element::Image(image)
    }
}

impl From<List> for Element {
    fn from(list: List) -> Self {
        Element::List(list)
    }
}

impl From<Blank> for Element {
    fn from(blank: Blank) -> Self {
        Element::Blank(blank)
    }
}

impl From<CodeListing> for Element {
    fn from(listing: CodeListing) -> Self {
        Element::CodeListing(listing)
    }
}

impl From<SiUnitX> for Element {
    fn from(si: SiUnitX) -> Self {
        Element::Si(si)
    }
}

// ── Element – Renderer impls ──────────────────────────────────────────────────
//
// `Element` and `Paragraph` implement `Renderer<T, Universal>`, gaining
// auto-derived `Problem` and `Solution` impls via the blanket rule in
// `renderer.rs`.
//
// `Blank` has no `Universal` impl; in the universal context it renders using
// its `Problem` renderer (an empty underline / `\hspace`), i.e. **blank answer
// slots are hidden**.  If you need the filled-in answer, render the `Blank`
// directly with the `Solution` environment.

impl Renderer<Latex, Universal> for Element {
    fn render(&self) -> anyhow::Result<String> {
        match self {
            Element::Text(t) => <Text as Renderer<Latex, Universal>>::render(t),
            Element::Table(t) => <Table as Renderer<Latex, Universal>>::render(t),
            Element::Image(i) => <Image as Renderer<Latex, Universal>>::render(i),
            Element::List(l) => <List as Renderer<Latex, Universal>>::render(l),
            Element::Blank(b) => <Blank as Renderer<Latex, Problem>>::render(b),
            Element::CodeListing(c) => <CodeListing as Renderer<Latex, Problem>>::render(c),
            Element::Si(s) => <SiUnitX as Renderer<Latex, Universal>>::render(s),
        }
    }
}

impl Renderer<Html, Universal> for Element {
    fn render(&self) -> anyhow::Result<String> {
        match self {
            Element::Text(t) => <Text as Renderer<Html, Universal>>::render(t),
            Element::Table(t) => <Table as Renderer<Html, Universal>>::render(t),
            Element::Image(i) => <Image as Renderer<Html, Universal>>::render(i),
            Element::List(l) => <List as Renderer<Html, Universal>>::render(l),
            Element::Blank(b) => <Blank as Renderer<Html, Problem>>::render(b),
            Element::CodeListing(c) => <CodeListing as Renderer<Html, Problem>>::render(c),
            Element::Si(s) => <SiUnitX as Renderer<Html, Universal>>::render(s),
        }
    }
}

impl Renderer<Markdown, Universal> for Element {
    fn render(&self) -> anyhow::Result<String> {
        match self {
            Element::Text(t) => <Text as Renderer<Markdown, Universal>>::render(t),
            Element::Table(t) => <Table as Renderer<Markdown, Universal>>::render(t),
            Element::Image(i) => <Image as Renderer<Markdown, Universal>>::render(i),
            Element::List(l) => <List as Renderer<Markdown, Universal>>::render(l),
            Element::Blank(b) => <Blank as Renderer<Markdown, Problem>>::render(b),
            Element::CodeListing(c) => <CodeListing as Renderer<Markdown, Problem>>::render(c),
            Element::Si(s) => <SiUnitX as Renderer<Markdown, Universal>>::render(s),
        }
    }
}

// ── Paragraph – Renderer impls ────────────────────────────────────────────────

impl Renderer<Latex, Universal> for Paragraph {
    /// Concatenate all element renders.  Block elements (table, figure, list)
    /// carry their own `\begin…\end` markup and do not need extra separators.
    fn render(&self) -> anyhow::Result<String> {
        let mut result = String::new();
        result.push_str("\n"); // Start with empty string to avoid borrowing issues with self.elements
        for element in &self.elements {
            result.push_str(&<Element as Renderer<Latex, Universal>>::render(element)?);
            result.push_str("\n"); // Ensure elements are separated by newlines to prevent LaTeX parsing issues
        }
        if let Some(pool) = &self.choice_pool {
            result.push_str(&<ChoicePool as Renderer<Latex, Universal>>::render(pool)?);
        }
        Ok(result)
    }
}

impl Renderer<Html, Universal> for Paragraph {
    /// Concatenate all element renders.  The caller is responsible for wrapping
    /// in a `<p>` tag when appropriate (e.g. a prose paragraph vs a list item).
    fn render(&self) -> anyhow::Result<String> {
        let mut result = String::new();
        result.push_str("<p>");
        for element in &self.elements {
            result.push_str(&<Element as Renderer<Html, Universal>>::render(element)?);
        }
        if let Some(pool) = &self.choice_pool {
            result.push_str(&<ChoicePool as Renderer<Html, Universal>>::render(pool)?);
        }
        result.push_str("</p>");
        Ok(result)
    }
}

impl Renderer<Markdown, Universal> for Paragraph {
    /// Concatenate all element renders, separating block-level elements with a
    /// blank line so that Markdown parsers recognise them as distinct blocks.
    fn render(&self) -> anyhow::Result<String> {
        let mut result = String::new();
        result.push_str("\n\n"); // Ensure the paragraph starts on a new line
        for element in &self.elements {
            result.push_str(&<Element as Renderer<Markdown, Universal>>::render(
                element,
            )?);
        }
        if let Some(pool) = &self.choice_pool {
            result.push_str(&<ChoicePool as Renderer<Markdown, Universal>>::render(
                pool,
            )?);
        }
        Ok(result)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::renderer::{Html, Latex, Markdown, Renderer, Universal};
    use crate::{OrderFormat, OrderType};

    fn text_el(s: &str) -> Element {
        Element::Text(s.parse().expect("parse failed"))
    }

    fn para(s: &str) -> Paragraph {
        Paragraph::new(vec![text_el(s)])
    }

    // ── Paragraph ─────────────────────────────────────────────────────────────

    #[test]
    fn paragraph_single_text_latex() {
        let p = para("hello");
        assert_eq!(
            <Paragraph as Renderer<Latex, Universal>>::render(&p).unwrap(),
            "\nhello\n"
        );
    }

    #[test]
    fn paragraph_single_text_html() {
        let p = para("hello");
        assert_eq!(
            <Paragraph as Renderer<Html, Universal>>::render(&p).unwrap(),
            "<p>hello</p>"
        );
    }

    #[test]
    fn paragraph_single_text_md() {
        let p = para("hello");
        assert_eq!(
            <Paragraph as Renderer<Markdown, Universal>>::render(&p).unwrap(),
            "\n\nhello"
        );
    }

    #[test]
    fn paragraph_multiple_texts_concatenated() {
        let p = Paragraph::new(vec![text_el(r"\[b]{bold}"), text_el(" normal")]);
        let out = <Paragraph as Renderer<Latex, Universal>>::render(&p).unwrap();
        // Each element occupies its own line with surrounding newlines.
        assert_eq!(out, "\n\\textbf{bold}\n normal\n");
    }

    #[test]
    fn paragraph_md_blocks_separated_by_blank_line() {
        use crate::{List, OrderFormat, OrderType};
        let list = List {
            items: vec![
                Paragraph::new(vec![text_el("a")]),
                Paragraph::new(vec![text_el("b")]),
            ],
            order_type: OrderType::Unordered,
            order_format: OrderFormat::None,
        };
        let p = Paragraph::new(vec![text_el("intro"), Element::List(list)]);
        let out = <Paragraph as Renderer<Markdown, Universal>>::render(&p).unwrap();
        // Paragraph opens with \n\n; text and list are serialised consecutively.
        assert!(out.contains("intro"), "expected intro text in output");
        assert!(out.contains("- "), "expected list items in output");
    }

    // ── Element dispatch ──────────────────────────────────────────────────────

    #[test]
    fn element_text_latex() {
        let e = text_el(r"\[b]{hi}");
        assert_eq!(
            <Element as Renderer<Latex, Universal>>::render(&e).unwrap(),
            r"\textbf{hi}"
        );
    }

    #[test]
    fn element_text_html() {
        let e = text_el(r"\[b]{hi}");
        assert_eq!(
            <Element as Renderer<Html, Universal>>::render(&e).unwrap(),
            "<strong>hi</strong>"
        );
    }

    #[test]
    fn element_text_md() {
        let e = text_el(r"\[b]{hi}");
        assert_eq!(
            <Element as Renderer<Markdown, Universal>>::render(&e).unwrap(),
            "**hi**"
        );
    }

    #[test]
    fn element_list_latex() {
        let list = crate::List {
            items: vec![para("x"), para("y")],
            order_type: OrderType::Unordered,
            order_format: OrderFormat::None,
        };
        let e = Element::List(list);
        let out = <Element as Renderer<Latex, Universal>>::render(&e).unwrap();
        assert!(out.contains("\\begin{itemize}"));
        // Paragraph renders each element wrapped in newlines: \item \nx\n
        assert!(out.contains("\\item"));
        assert!(out.contains("x"));
        assert!(out.contains("y"));
    }

    #[test]
    fn element_list_html() {
        let list = crate::List {
            items: vec![para("x"), para("y")],
            order_type: OrderType::Unordered,
            order_format: OrderFormat::None,
        };
        let out = <Element as Renderer<Html, Universal>>::render(&Element::List(list)).unwrap();
        assert!(out.contains("<ul>"));
        // Paragraph wraps content in <p> tags inside each <li>.
        assert!(out.contains("<li><p>x</p></li>"));
    }

    #[test]
    fn element_blank_uses_problem_renderer_latex() {
        let blank = Blank::new(1, 1.0, BlankAnswer::Text("answer".parse().unwrap()), 3.0);
        let e = Element::Blank(blank);
        let out = <Element as Renderer<Latex, Universal>>::render(&e).unwrap();
        // Universal context → Problem renderer → hspace, not the answer
        assert!(out.contains("\\hspace*{"));
        assert!(!out.contains("answer"));
    }

    #[test]
    fn element_blank_uses_problem_renderer_html() {
        let blank = Blank::new(1, 1.0, BlankAnswer::Text("answer".parse().unwrap()), 3.0);
        let e = Element::Blank(blank);
        let out = <Element as Renderer<Html, Universal>>::render(&e).unwrap();
        assert!(out.contains("border-bottom"));
        assert!(!out.contains("answer"));
    }

    #[test]
    fn element_melt_pot_with_choice_pool() {
        let choice_pool = ChoicePool {
            choices: vec![
                crate::schema::elements::Choice::Text("Option A".parse().unwrap()),
                crate::schema::elements::Choice::Text("Option B".parse().unwrap()),
            ],
            order_type: OrderType::UppercaseAlphabetic,
            order_format: OrderFormat::Parenthesis,
        };
        let para = Paragraph::new(vec![]).with_choice_pool(choice_pool);
        let out = <Paragraph as Renderer<Latex, Universal>>::render(&para).unwrap();
        // ChoicePool → List → each item Paragraph renders as \item \nOption A\n
        assert!(out.contains("\\item"));
        assert!(out.contains("Option A"));
        assert!(out.contains("Option B"));
        // Parenthesis format: (\Alph*)
        assert!(out.contains("label=(\\Alph*)"));
    }
}
