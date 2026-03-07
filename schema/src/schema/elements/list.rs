use super::element::Paragraph;
use crate::schema::renderer::{Html, Latex, Markdown, Renderer, Universal};

/// The counter / bullet style for a list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderType {
    UppercaseAlphabetic,
    LowercaseAlphabetic,
    UppercaseRoman,
    LowercaseRoman,
    Decimal,
    Unordered,
}

impl OrderType {
    pub fn process(&self, n: usize) -> String {
        match self {
            OrderType::UppercaseAlphabetic => to_alphabetic(n, true),
            OrderType::LowercaseAlphabetic => to_alphabetic(n, false),
            OrderType::UppercaseRoman => to_roman(n, true),
            OrderType::LowercaseRoman => to_roman(n, false),
            OrderType::Decimal => n.to_string(),
            OrderType::Unordered => "".to_string(),
        }
    }
}

/// Converts a number to alphabetic (A, B, ..., Z, AA, AB, ...)
fn to_alphabetic(n: usize, uppercase: bool) -> String {
    let mut n = n;
    let mut result = String::new();

    loop {
        if n == 0 {
            break;
        }
        n -= 1;
        let ch = (b'A' + (n % 26) as u8) as char;
        result.insert(
            0,
            if uppercase {
                ch
            } else {
                ch.to_lowercase().next().unwrap()
            },
        );
        n /= 26;
    }

    if result.is_empty() {
        result.push(if uppercase { 'A' } else { 'a' });
    }

    result
}

/// Converts a number to Roman numerals
fn to_roman(n: usize, uppercase: bool) -> String {
    if n == 0 {
        return String::from("0");
    }

    const ROMAN_NUMERALS: &[(usize, &str)] = &[
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];

    let mut result = String::new();
    let mut remaining = n;

    for &(value, symbol) in ROMAN_NUMERALS {
        while remaining >= value {
            result.push_str(symbol);
            remaining -= value;
        }
    }

    if uppercase {
        result
    } else {
        result.to_lowercase()
    }
}

/// How the counter is wrapped / punctuated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderFormat {
    /// `1.`  `a.`  etc.
    Period,
    /// `(1)` `(a)` etc.
    Parenthesis,
    /// `1)`  `a)`  etc.
    RightParenthesis,
    /// `1`   `a`   etc.
    None,
}

#[derive(Clone, Debug)]
pub struct List {
    pub items: Vec<Paragraph>,
    pub order_type: OrderType,
    pub order_format: OrderFormat,
}

impl OrderType {
    /// LaTeX `enumitem` counter command, e.g. `\arabic`, `\alph`.
    /// Returns `None` for `Unordered`.
    pub(crate) fn latex_counter(&self) -> Option<&'static str> {
        match self {
            OrderType::Decimal => Some("\\arabic"),
            OrderType::LowercaseAlphabetic => Some("\\alph"),
            OrderType::UppercaseAlphabetic => Some("\\Alph"),
            OrderType::LowercaseRoman => Some("\\roman"),
            OrderType::UppercaseRoman => Some("\\Roman"),
            OrderType::Unordered => None,
        }
    }

    /// HTML `<ol type="…">` attribute value.  Returns `None` for `Unordered`.
    pub(crate) fn html_type(&self) -> Option<&'static str> {
        match self {
            OrderType::Decimal => Some("1"),
            OrderType::LowercaseAlphabetic => Some("a"),
            OrderType::UppercaseAlphabetic => Some("A"),
            OrderType::LowercaseRoman => Some("i"),
            OrderType::UppercaseRoman => Some("I"),
            OrderType::Unordered => None,
        }
    }
}

impl OrderFormat {
    /// Wrap `counter_cmd` (e.g. `\\arabic*`) according to this format.
    pub(crate) fn wrap_latex(&self, counter: &str) -> String {
        match self {
            OrderFormat::Period => format!("{}.", counter),
            OrderFormat::Parenthesis => format!("({})", counter),
            OrderFormat::RightParenthesis => format!("{})", counter),
            OrderFormat::None => counter.to_owned(),
        }
    }
}

// ── LaTeX ─────────────────────────────────────────────────────────────────────

impl Renderer<Latex, Universal> for List {
    /// Render the list as a LaTeX `itemize` or `enumerate` environment.
    ///
    /// - `Unordered` → `\begin{itemize} … \end{itemize}`
    /// - All ordered types → `\begin{enumerate}[label=…]` using the
    ///   `enumitem` package (already pulled in by the project preamble).
    ///
    /// The `label=` string is built from the [`OrderType`] counter command and
    /// the [`OrderFormat`] punctuation wrapper, e.g.:
    ///
    /// | Type               | Format           | `label=`          |
    /// |--------------------|------------------|-------------------|
    /// | Decimal            | Period           | `\arabic*.`       |
    /// | LowercaseAlphabetic| RightParenthesis | `\alph*)`         |
    /// | UppercaseRoman     | Parenthesis      | `(\Roman*)`       |
    fn render(&self) -> anyhow::Result<String> {
        let mut out = String::new();

        if self.order_type == OrderType::Unordered {
            out.push_str("\\begin{itemize}\n");
            for item in &self.items {
                let text = <Paragraph as Renderer<Latex, Universal>>::render(item)?;
                out.push_str(&format!("  \\item {}\n", text));
            }
            out.push_str("\\end{itemize}");
        } else {
            let counter_cmd = self.order_type.latex_counter().unwrap();
            // enumitem uses `*` as a placeholder for the current counter value.
            let label = self.order_format.wrap_latex(&format!("{}*", counter_cmd));
            out.push_str(&format!("\\begin{{enumerate}}[label={}]\n", label));
            for item in &self.items {
                let text = <Paragraph as Renderer<Latex, Universal>>::render(item)?;
                out.push_str(&format!("  \\item {}\n", text));
            }
            out.push_str("\\end{enumerate}");
        }

        Ok(out)
    }
}

// ── HTML ──────────────────────────────────────────────────────────────────────

impl Renderer<Html, Universal> for List {
    /// Render the list as an HTML `<ul>` or `<ol>` element.
    ///
    /// - `Unordered` → `<ul>`
    /// - Ordered types → `<ol type="…">` where `type` follows the HTML spec
    ///   (`1`, `a`, `A`, `i`, `I`).
    /// - [`OrderFormat`] variants other than `Period` are expressed via an
    ///   inline `list-style-type` CSS counter on the `<ol>` element:
    ///
    /// | Format           | CSS `list-style-type`          |
    /// |------------------|-------------------------------|
    /// | Period           | *(browser default)*            |
    /// | RightParenthesis | `"\2e "` replaced by `"\\29 "` |
    /// | Parenthesis      | counters with open+close paren |
    /// | None             | `none` + `padding-left:0`      |
    fn render(&self) -> anyhow::Result<String> {
        let mut out = String::new();

        if self.order_type == OrderType::Unordered {
            out.push_str("<ul>\n");
            for item in &self.items {
                let text = <Paragraph as Renderer<Html, Universal>>::render(item)?;
                out.push_str(&format!("  <li>{}</li>\n", text));
            }
            out.push_str("</ul>");
        } else {
            let type_attr = self.order_type.html_type().unwrap();

            // CSS override for non-Period formats.
            let style_attr = match self.order_format {
                OrderFormat::Period => String::new(),
                OrderFormat::RightParenthesis => {
                    format!(" style=\"list-style-type: '\\29 '\"")
                }
                OrderFormat::Parenthesis => {
                    // No single `type` value covers this; use a CSS counter.
                    format!(
                        " style=\"list-style-type: '(' counter(list-item, {type_attr}) ')'\"",
                        type_attr = match self.order_type.html_type().unwrap() {
                            "1" => "decimal",
                            "a" => "lower-alpha",
                            "A" => "upper-alpha",
                            "i" => "lower-roman",
                            "I" => "upper-roman",
                            other => other,
                        }
                    )
                }
                OrderFormat::None => {
                    format!(" style=\"list-style-type:none;padding-left:0\"")
                }
            };

            out.push_str(&format!("<ol type=\"{type_attr}\"{style_attr}>\n"));
            for item in &self.items {
                let text = <Paragraph as Renderer<Html, Universal>>::render(item)?;
                out.push_str(&format!("  <li>{}</li>\n", text));
            }
            out.push_str("</ol>");
        }

        Ok(out)
    }
}

// ── Markdown ──────────────────────────────────────────────────────────────────

impl Renderer<Markdown, Universal> for List {
    /// Render the list as Markdown.
    ///
    /// - `Unordered` → `- item` (GFM bullet list)
    /// - `Decimal` + `Period` → `1. item` (GFM ordered list; browsers
    ///   auto-number so every item can be `1.`)
    /// - All other type/format combinations → falls back to the HTML renderer,
    ///   since CommonMark has no syntax for alphabetic or roman-numeral lists.
    fn render(&self) -> anyhow::Result<String> {
        match (&self.order_type, &self.order_format) {
            (OrderType::Unordered, _) => {
                let mut out = String::new();
                for item in &self.items {
                    let text = <Paragraph as Renderer<Markdown, Universal>>::render(item)?;
                    out.push_str(&format!("- {}\n", text));
                }
                if out.ends_with('\n') {
                    out.pop();
                }
                Ok(out)
            }
            (OrderType::Decimal, OrderFormat::Period) => {
                let mut out = String::new();
                for item in &self.items {
                    let text = <Paragraph as Renderer<Markdown, Universal>>::render(item)?;
                    out.push_str(&format!("1. {}\n", text));
                }
                if out.ends_with('\n') {
                    out.pop();
                }
                Ok(out)
            }
            // All other combinations have no native Markdown encoding.
            _ => <List as Renderer<Html, Universal>>::render(self),
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::super::element::Element;
    use super::*;
    use crate::Text;
    use crate::schema::renderer::{Html, Latex, Markdown, Renderer, Universal};

    fn para(s: &str) -> Paragraph {
        let t: Text = s.parse().expect("text parse failed");
        Paragraph::new(vec![Element::Text(t)])
    }

    fn unordered(items: &[&str]) -> List {
        List {
            items: items.iter().map(|s| para(s)).collect(),
            order_type: OrderType::Unordered,
            order_format: OrderFormat::None,
        }
    }

    fn ordered(items: &[&str], ty: OrderType, fmt: OrderFormat) -> List {
        List {
            items: items.iter().map(|s| para(s)).collect(),
            order_type: ty,
            order_format: fmt,
        }
    }

    // ── LaTeX ─────────────────────────────────────────────────────────────────

    #[test]
    fn latex_unordered() {
        let l = unordered(&["foo", "bar"]);
        let out = <List as Renderer<Latex, Universal>>::render(&l).unwrap();
        assert!(out.contains("\\begin{itemize}"));
        assert!(out.contains("\\end{itemize}"));
        // Paragraph wraps each element in newlines: \item \nfoo\n
        assert!(out.contains("\\item"));
        assert!(out.contains("foo"));
        assert!(out.contains("bar"));
    }

    #[test]
    fn latex_decimal_period() {
        let l = ordered(&["one", "two"], OrderType::Decimal, OrderFormat::Period);
        let out = <List as Renderer<Latex, Universal>>::render(&l).unwrap();
        assert!(out.contains("\\begin{enumerate}[label=\\arabic*.]"));
        assert!(out.contains("\\item"));
        assert!(out.contains("one"));
    }

    #[test]
    fn latex_decimal_parenthesis() {
        let l = ordered(&["x"], OrderType::Decimal, OrderFormat::Parenthesis);
        let out = <List as Renderer<Latex, Universal>>::render(&l).unwrap();
        assert!(out.contains("label=(\\arabic*)"));
    }

    #[test]
    fn latex_decimal_right_paren() {
        let l = ordered(&["x"], OrderType::Decimal, OrderFormat::RightParenthesis);
        let out = <List as Renderer<Latex, Universal>>::render(&l).unwrap();
        assert!(out.contains("label=\\arabic*)"));
    }

    #[test]
    fn latex_decimal_none() {
        let l = ordered(&["x"], OrderType::Decimal, OrderFormat::None);
        let out = <List as Renderer<Latex, Universal>>::render(&l).unwrap();
        assert!(out.contains("label=\\arabic*]"));
    }

    #[test]
    fn latex_lowercase_alpha() {
        let l = ordered(
            &["a", "b"],
            OrderType::LowercaseAlphabetic,
            OrderFormat::Period,
        );
        let out = <List as Renderer<Latex, Universal>>::render(&l).unwrap();
        assert!(out.contains("label=\\alph*."));
        assert!(out.contains("\\end{enumerate}"));
    }

    #[test]
    fn latex_uppercase_alpha() {
        let l = ordered(&["a"], OrderType::UppercaseAlphabetic, OrderFormat::Period);
        let out = <List as Renderer<Latex, Universal>>::render(&l).unwrap();
        assert!(out.contains("label=\\Alph*."));
    }

    #[test]
    fn latex_lowercase_roman() {
        let l = ordered(
            &["a"],
            OrderType::LowercaseRoman,
            OrderFormat::RightParenthesis,
        );
        let out = <List as Renderer<Latex, Universal>>::render(&l).unwrap();
        assert!(out.contains("label=\\roman*)"));
    }

    #[test]
    fn latex_uppercase_roman() {
        let l = ordered(&["a"], OrderType::UppercaseRoman, OrderFormat::Parenthesis);
        let out = <List as Renderer<Latex, Universal>>::render(&l).unwrap();
        assert!(out.contains("label=(\\Roman*)"));
    }

    #[test]
    fn latex_formatted_item_content() {
        let l = List {
            items: vec![para(r"\[b]{bold item}")],
            order_type: OrderType::Unordered,
            order_format: OrderFormat::None,
        };
        let out = <List as Renderer<Latex, Universal>>::render(&l).unwrap();
        assert!(out.contains("\\textbf{bold item}"));
    }

    // ── HTML ──────────────────────────────────────────────────────────────────

    #[test]
    fn html_unordered() {
        let l = unordered(&["alpha", "beta"]);
        let out = <List as Renderer<Html, Universal>>::render(&l).unwrap();
        assert!(out.contains("<ul>"));
        assert!(out.contains("</ul>"));
        // Paragraph wraps content in <p> tags inside each <li>.
        assert!(out.contains("<li><p>alpha</p></li>"));
        assert!(out.contains("<li><p>beta</p></li>"));
    }

    #[test]
    fn html_decimal_period() {
        let l = ordered(&["one", "two"], OrderType::Decimal, OrderFormat::Period);
        let out = <List as Renderer<Html, Universal>>::render(&l).unwrap();
        assert!(out.contains(r#"<ol type="1">"#));
        // No extra style attr for Period.
        assert!(!out.contains("style"));
    }

    #[test]
    fn html_decimal_right_parenthesis() {
        let l = ordered(&["x"], OrderType::Decimal, OrderFormat::RightParenthesis);
        let out = <List as Renderer<Html, Universal>>::render(&l).unwrap();
        assert!(out.contains(r#"type="1""#));
        assert!(out.contains("style="));
    }

    #[test]
    fn html_decimal_none() {
        let l = ordered(&["x"], OrderType::Decimal, OrderFormat::None);
        let out = <List as Renderer<Html, Universal>>::render(&l).unwrap();
        assert!(out.contains("list-style-type:none"));
    }

    #[test]
    fn html_lowercase_alpha() {
        let l = ordered(
            &["a", "b"],
            OrderType::LowercaseAlphabetic,
            OrderFormat::Period,
        );
        let out = <List as Renderer<Html, Universal>>::render(&l).unwrap();
        assert!(out.contains(r#"type="a""#));
    }

    #[test]
    fn html_uppercase_alpha() {
        let l = ordered(&["a"], OrderType::UppercaseAlphabetic, OrderFormat::Period);
        let out = <List as Renderer<Html, Universal>>::render(&l).unwrap();
        assert!(out.contains(r#"type="A""#));
    }

    #[test]
    fn html_lowercase_roman() {
        let l = ordered(&["a"], OrderType::LowercaseRoman, OrderFormat::Period);
        let out = <List as Renderer<Html, Universal>>::render(&l).unwrap();
        assert!(out.contains(r#"type="i""#));
    }

    #[test]
    fn html_uppercase_roman() {
        let l = ordered(&["a"], OrderType::UppercaseRoman, OrderFormat::Period);
        let out = <List as Renderer<Html, Universal>>::render(&l).unwrap();
        assert!(out.contains(r#"type="I""#));
    }

    #[test]
    fn html_items_rendered_correctly() {
        let l = unordered(&["hello", "world"]);
        let out = <List as Renderer<Html, Universal>>::render(&l).unwrap();
        assert!(out.contains("<li><p>hello</p></li>"));
        assert!(out.contains("<li><p>world</p></li>"));
    }

    // ── Markdown ──────────────────────────────────────────────────────────────

    #[test]
    fn md_unordered() {
        let l = unordered(&["foo", "bar", "baz"]);
        let out = <List as Renderer<Markdown, Universal>>::render(&l).unwrap();
        // Paragraph renders as \n\ncontent; list items are "- \n\nfoo".
        assert!(out.contains("- "));
        assert!(out.contains("foo"));
        assert!(out.contains("bar"));
        assert!(out.contains("baz"));
    }

    #[test]
    fn md_decimal_period() {
        let l = ordered(&["one", "two"], OrderType::Decimal, OrderFormat::Period);
        let out = <List as Renderer<Markdown, Universal>>::render(&l).unwrap();
        // Paragraph renders as \n\ncontent; list items are "1. \n\none".
        assert!(out.contains("1. "));
        assert!(out.contains("one"));
        assert!(out.contains("two"));
    }

    #[test]
    fn md_alpha_falls_back_to_html() {
        let l = ordered(&["x"], OrderType::LowercaseAlphabetic, OrderFormat::Period);
        let out = <List as Renderer<Markdown, Universal>>::render(&l).unwrap();
        assert!(out.contains("<ol"));
    }

    #[test]
    fn md_decimal_parenthesis_falls_back_to_html() {
        let l = ordered(&["x"], OrderType::Decimal, OrderFormat::Parenthesis);
        let out = <List as Renderer<Markdown, Universal>>::render(&l).unwrap();
        assert!(out.contains("<ol"));
    }

    #[test]
    fn md_roman_falls_back_to_html() {
        let l = ordered(&["x"], OrderType::LowercaseRoman, OrderFormat::Period);
        let out = <List as Renderer<Markdown, Universal>>::render(&l).unwrap();
        assert!(out.contains("<ol"));
    }

    #[test]
    fn md_formatted_item_content() {
        let l = List {
            items: vec![para(r"\[b]{important}")],
            order_type: OrderType::Unordered,
            order_format: OrderFormat::None,
        };
        let out = <List as Renderer<Markdown, Universal>>::render(&l).unwrap();
        // Paragraph renders as \n\n**important**; list item is "- \n\n**important**".
        assert_eq!(out, "- \n\n**important**");
    }
}
