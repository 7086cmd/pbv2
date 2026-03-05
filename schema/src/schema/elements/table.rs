use std::collections::{HashMap, HashSet};

use crate::Text;
use crate::schema::renderer::{Html, Latex, Markdown, Renderer, Universal};

pub struct Cell {
    pub content: Text,
    pub row_span: usize,
    pub col_span: usize,

    /// Whether this cell is a header cell (`<th>` in HTML; bold wrapper in LaTeX).
    pub header: bool,
}

pub struct Table {
    pub rows: usize,
    pub cols: usize,

    /// Map from `(row, col)` to [`Cell`].  Row and column indices are 0-based.
    pub cells: HashMap<(usize, usize), Cell>,
}

impl Table {
    /// Returns `true` if any cell spans more than one row or column.
    pub fn has_merged_cells(&self) -> bool {
        self.cells
            .values()
            .any(|c| c.row_span > 1 || c.col_span > 1)
    }

    /// Collects every `(row, col)` position that is *covered* by a spanning
    /// cell whose origin lies elsewhere.  The origin position itself is **not**
    /// included in the returned set.
    fn covered_positions(&self) -> HashSet<(usize, usize)> {
        let mut covered = HashSet::new();
        for (&(row, col), cell) in &self.cells {
            for r in row..row + cell.row_span {
                for c in col..col + cell.col_span {
                    if (r, c) != (row, col) {
                        covered.insert((r, c));
                    }
                }
            }
        }
        covered
    }
}

// ── LaTeX ─────────────────────────────────────────────────────────────────────

impl Renderer<Latex, Universal> for Table {
    /// Render the table as a LaTeX `tabular` environment.
    ///
    /// - Column spec uses `|l|` repeated for each column.
    /// - Column spans → `\multicolumn{n}{|l|}{…}`.
    /// - Row spans → `\multirow{n}{*}{…}` (requires the `multirow` package).
    /// - Header cells have their rendered text wrapped in `\textbf{…}`.
    /// - Cells that are covered by a row-spanning cell from a previous row are
    ///   emitted as empty cells so that the `&` separators stay aligned.
    fn render(&self) -> anyhow::Result<String> {
        let covered = self.covered_positions();

        // Build column spec: |l|l|...|l|
        let col_spec = format!("|{}|", vec!["l"; self.cols].join("|"));

        let mut out = String::new();
        out.push_str(&format!("\\begin{{tabular}}{{{}}}\n", col_spec));
        out.push_str("\\hline\n");

        for row in 0..self.rows {
            let mut cells_in_row: Vec<String> = Vec::new();
            let mut col = 0;

            while col < self.cols {
                if covered.contains(&(row, col)) {
                    // This position is owned by a row-spanning cell above.
                    // Emit an empty cell to keep column alignment correct.
                    cells_in_row.push(String::new());
                    col += 1;
                    continue;
                }

                let cell = match self.cells.get(&(row, col)) {
                    Some(c) => c,
                    None => {
                        cells_in_row.push(String::new());
                        col += 1;
                        continue;
                    }
                };

                let text = <Text as Renderer<Latex, Universal>>::render(&cell.content)?;
                let text = if cell.header {
                    format!("\\textbf{{{}}}", text)
                } else {
                    text
                };

                let seg = match (cell.col_span > 1, cell.row_span > 1) {
                    (true, true) => format!(
                        "\\multicolumn{{{}}}{{|l|}}{{\\multirow{{{}}}{{*}}{{{}}}}}",
                        cell.col_span, cell.row_span, text
                    ),
                    (true, false) => {
                        format!("\\multicolumn{{{}}}{{|l|}}{{{}}}", cell.col_span, text)
                    }
                    (false, true) => {
                        format!("\\multirow{{{}}}{{*}}{{{}}}", cell.row_span, text)
                    }
                    (false, false) => text,
                };

                cells_in_row.push(seg);
                col += cell.col_span;
            }

            out.push_str(&cells_in_row.join(" & "));
            out.push_str(" \\\\\n\\hline\n");
        }

        out.push_str("\\end{tabular}");
        Ok(out)
    }
}

// ── HTML ──────────────────────────────────────────────────────────────────────

impl Renderer<Html, Universal> for Table {
    /// Render the table as an HTML `<table>` element.
    ///
    /// - Header cells (`cell.header == true`) → `<th>`, others → `<td>`.
    /// - `colspan` / `rowspan` attributes are added when the span is > 1.
    /// - Positions covered by a spanning cell are skipped entirely (the browser
    ///   derives them from the `colspan`/`rowspan` of the origin cell).
    fn render(&self) -> anyhow::Result<String> {
        let covered = self.covered_positions();

        let mut out = String::from("<table>\n");

        for row in 0..self.rows {
            out.push_str("  <tr>\n");
            let mut col = 0;

            while col < self.cols {
                if covered.contains(&(row, col)) {
                    col += 1;
                    continue;
                }

                let cell = match self.cells.get(&(row, col)) {
                    Some(c) => c,
                    None => {
                        let tag = "td";
                        out.push_str(&format!("    <{0}></{0}>\n", tag));
                        col += 1;
                        continue;
                    }
                };

                let tag = if cell.header { "th" } else { "td" };

                let mut attrs = String::new();
                if cell.col_span > 1 {
                    attrs.push_str(&format!(" colspan=\"{}\"", cell.col_span));
                }
                if cell.row_span > 1 {
                    attrs.push_str(&format!(" rowspan=\"{}\"", cell.row_span));
                }

                let text = <Text as Renderer<Html, Universal>>::render(&cell.content)?;
                out.push_str(&format!("    <{0}{1}>{2}</{0}>\n", tag, attrs, text));

                col += cell.col_span;
            }

            out.push_str("  </tr>\n");
        }

        out.push_str("</table>");
        Ok(out)
    }
}

// ── Markdown ──────────────────────────────────────────────────────────────────

impl Renderer<Markdown, Universal> for Table {
    /// Render the table as Markdown.
    ///
    /// - If the table has **no** merged cells, a GFM pipe-table is produced.
    ///   Row 0 is always the header row (with `---` separators beneath it).
    ///   Cells marked `header = true` have their content bolded.
    /// - If the table has merged cells, the HTML renderer is used as a
    ///   fallback, since pipe-tables cannot express `colspan`/`rowspan`.
    fn render(&self) -> anyhow::Result<String> {
        if self.has_merged_cells() {
            // Fall back to HTML for tables with spanning cells.
            return <Table as Renderer<Html, Universal>>::render(self);
        }

        let mut out = String::new();

        for row in 0..self.rows {
            let mut cells: Vec<String> = Vec::new();

            for col in 0..self.cols {
                let text = match self.cells.get(&(row, col)) {
                    Some(cell) => {
                        let rendered =
                            <Text as Renderer<Markdown, Universal>>::render(&cell.content)?;
                        // Bold header cells that aren't already styled.
                        if cell.header {
                            format!("**{}**", rendered)
                        } else {
                            rendered
                        }
                    }
                    None => String::new(),
                };
                // Escape any pipe characters inside the cell content.
                cells.push(text.replace('|', "\\|"));
            }

            out.push('|');
            out.push_str(&cells.join(" | "));
            out.push_str(" |\n");

            // After the first row, emit the separator line.
            if row == 0 {
                out.push('|');
                out.push_str(&vec!["---"; self.cols].join(" | "));
                out.push_str(" |\n");
            }
        }

        // Strip trailing newline for consistency with the other renderers.
        if out.ends_with('\n') {
            out.pop();
        }

        Ok(out)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::renderer::{Html, Latex, Markdown, Renderer, Universal};

    fn text(s: &str) -> Text {
        s.parse().expect("text parse failed")
    }

    fn cell(s: &str) -> Cell {
        Cell {
            content: text(s),
            row_span: 1,
            col_span: 1,
            header: false,
        }
    }

    fn header_cell(s: &str) -> Cell {
        Cell {
            content: text(s),
            row_span: 1,
            col_span: 1,
            header: true,
        }
    }

    /// Build a simple `rows × cols` table from a flat slice of cells
    /// (row-major order, no spanning).
    fn simple_table(rows: usize, cols: usize, cells: Vec<Cell>) -> Table {
        let mut map = HashMap::new();
        for (i, c) in cells.into_iter().enumerate() {
            map.insert((i / cols, i % cols), c);
        }
        Table {
            rows,
            cols,
            cells: map,
        }
    }

    // ── has_merged_cells ──────────────────────────────────────────────────────

    #[test]
    fn no_merged_cells() {
        let t = simple_table(2, 2, vec![cell("a"), cell("b"), cell("c"), cell("d")]);
        assert!(!t.has_merged_cells());
    }

    #[test]
    fn colspan_detected() {
        let mut t = simple_table(1, 2, vec![cell("a"), cell("b")]);
        t.cells.get_mut(&(0, 0)).unwrap().col_span = 2;
        assert!(t.has_merged_cells());
    }

    #[test]
    fn rowspan_detected() {
        let mut t = simple_table(2, 1, vec![cell("a"), cell("b")]);
        t.cells.get_mut(&(0, 0)).unwrap().row_span = 2;
        assert!(t.has_merged_cells());
    }

    // ── LaTeX ─────────────────────────────────────────────────────────────────

    #[test]
    fn latex_simple_2x2() {
        let t = simple_table(
            2,
            2,
            vec![header_cell("A"), header_cell("B"), cell("1"), cell("2")],
        );
        let out = <Table as Renderer<Latex, Universal>>::render(&t).unwrap();
        assert!(out.contains("\\begin{tabular}{|l|l|}"));
        assert!(out.contains("\\textbf{A}"));
        assert!(out.contains("\\textbf{B}"));
        assert!(out.contains("\\end{tabular}"));
        assert!(out.contains("\\hline"));
    }

    #[test]
    fn latex_colspan() {
        let mut cells = HashMap::new();
        cells.insert(
            (0, 0),
            Cell {
                content: text("wide"),
                row_span: 1,
                col_span: 2,
                header: false,
            },
        );
        cells.insert((1, 0), cell("L"));
        cells.insert((1, 1), cell("R"));
        let t = Table {
            rows: 2,
            cols: 2,
            cells,
        };
        let out = <Table as Renderer<Latex, Universal>>::render(&t).unwrap();
        assert!(out.contains("\\multicolumn{2}{|l|}{wide}"));
    }

    #[test]
    fn latex_rowspan() {
        let mut cells = HashMap::new();
        cells.insert(
            (0, 0),
            Cell {
                content: text("tall"),
                row_span: 2,
                col_span: 1,
                header: false,
            },
        );
        cells.insert((0, 1), cell("X"));
        cells.insert((1, 1), cell("Y"));
        let t = Table {
            rows: 2,
            cols: 2,
            cells,
        };
        let out = <Table as Renderer<Latex, Universal>>::render(&t).unwrap();
        assert!(out.contains("\\multirow{2}{*}{tall}"));
    }

    // ── HTML ──────────────────────────────────────────────────────────────────

    #[test]
    fn html_simple_2x2() {
        let t = simple_table(
            2,
            2,
            vec![header_cell("A"), header_cell("B"), cell("1"), cell("2")],
        );
        let out = <Table as Renderer<Html, Universal>>::render(&t).unwrap();
        assert!(out.contains("<table>"));
        assert!(out.contains("<th>A</th>"));
        assert!(out.contains("<th>B</th>"));
        assert!(out.contains("<td>1</td>"));
        assert!(out.contains("<td>2</td>"));
        assert!(out.contains("</table>"));
    }

    #[test]
    fn html_colspan() {
        let mut cells = HashMap::new();
        cells.insert(
            (0, 0),
            Cell {
                content: text("wide"),
                row_span: 1,
                col_span: 2,
                header: false,
            },
        );
        cells.insert((1, 0), cell("L"));
        cells.insert((1, 1), cell("R"));
        let t = Table {
            rows: 2,
            cols: 2,
            cells,
        };
        let out = <Table as Renderer<Html, Universal>>::render(&t).unwrap();
        assert!(out.contains(r#"colspan="2""#));
        assert!(!out.contains(r#"rowspan"#));
    }

    #[test]
    fn html_rowspan() {
        let mut cells = HashMap::new();
        cells.insert(
            (0, 0),
            Cell {
                content: text("tall"),
                row_span: 2,
                col_span: 1,
                header: false,
            },
        );
        cells.insert((0, 1), cell("X"));
        cells.insert((1, 1), cell("Y"));
        let t = Table {
            rows: 2,
            cols: 2,
            cells,
        };
        let out = <Table as Renderer<Html, Universal>>::render(&t).unwrap();
        assert!(out.contains(r#"rowspan="2""#));
        // The covered cell (1,0) must NOT produce a <td> element.
        let td_count = out.matches("<td").count();
        assert_eq!(td_count, 3); // tall, X, Y — not a phantom 4th
    }

    // ── Markdown ──────────────────────────────────────────────────────────────

    #[test]
    fn md_simple_pipe_table() {
        let t = simple_table(
            3,
            2,
            vec![
                header_cell("Name"),
                header_cell("Value"),
                cell("foo"),
                cell("42"),
                cell("bar"),
                cell("7"),
            ],
        );
        let out = <Table as Renderer<Markdown, Universal>>::render(&t).unwrap();
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 4);
        assert!(lines[0].starts_with('|'));
        // Separator row
        assert!(lines[1].contains("---"));
        // Header cells bolded
        assert!(lines[0].contains("**Name**"));
        assert!(lines[0].contains("**Value**"));
    }

    #[test]
    fn md_falls_back_to_html_for_merged_cells() {
        let mut cells = HashMap::new();
        cells.insert(
            (0, 0),
            Cell {
                content: text("wide"),
                row_span: 1,
                col_span: 2,
                header: false,
            },
        );
        cells.insert((1, 0), cell("L"));
        cells.insert((1, 1), cell("R"));
        let t = Table {
            rows: 2,
            cols: 2,
            cells,
        };
        let out = <Table as Renderer<Markdown, Universal>>::render(&t).unwrap();
        assert!(out.contains("<table>"));
    }

    #[test]
    fn md_pipe_in_cell_is_escaped() {
        let t = simple_table(2, 1, vec![header_cell("h"), cell("a|b")]);
        let out = <Table as Renderer<Markdown, Universal>>::render(&t).unwrap();
        assert!(out.contains("a\\|b"));
    }
}
