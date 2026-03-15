use uuid::Uuid;

use crate::{
    DocumentClass, ElementalQuestion, LatexBuilder,
    schema::{
        papers::cirriculum::{Cirriculum, Subject},
        renderer::{Html, Latex, Markdown, Problem, Renderer},
    },
};

#[derive(Debug, Clone)]
pub struct Paper {
    pub id: Uuid,
    pub name: String,
    pub subject: Subject,
    pub cirriculum: Cirriculum,
    pub abbreviation: String,
    pub paper_type: PaperType,

    pub instructions: Vec<Instruction>,

    /// The year the paper was administered, if applicable. For mock papers, this field may be null.
    pub year: Option<i32>,

    pub content: Vec<ElementalQuestion>
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PaperType {
    /// A paper that has been administered to students and has real student responses. This is the most common type of paper and is used for summative assessment.
    Actual,
    /// A paper that is designed to be similar to an actual paper but has not been administered to students and does not have real student responses. This type of paper is often used for formative assessment, practice, and drill.
    Mock,
    /// A paper might be used for, e.g., midterms, final exams, that is not standardized and is only used within a specific school or district. This type of paper is often used for formative assessment, practice, and drill.
    Practice,
    /// Not in assessment at all.
    Drill
}

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    Constants(Vec<(String, f32)>),
    Formulae(Vec<String>),
    RelativeAtomicMasses(Vec<(String, f32)>),
}

impl Instruction {
    pub fn is_empty(&self) -> bool {
        match self {
            Instruction::Constants(constants) => constants.is_empty(),
            Instruction::Formulae(formulae) => formulae.is_empty(),
            Instruction::RelativeAtomicMasses(relative_atomic_masses) => relative_atomic_masses.is_empty(),
        }
    }
}

// ── Paper – Renderer impls ────────────────────────────────────────────────────

impl Renderer<Latex, Problem> for Paper {
    /// Renders the paper as a complete XeLaTeX document.
    ///
    /// Layout:
    /// 1. Title: `\name (\abbreviation) – \year` (year omitted when `None`).
    /// 2. Instructions section (`\section*{Instructions}`) listing each
    ///    non-empty [`Instruction`] variant as a description-list entry.
    /// 3. Problems (`\section*{Problems}`) with each [`ElementalQuestion`]
    ///    rendered in sequence and separated by `\bigskip`.
    fn render(&self) -> anyhow::Result<String> {
        let mut builder = LatexBuilder::new(DocumentClass::Article { toc: false });

        // ── title ──────────────────────────────────────────────────────────
        let title = match self.year {
            Some(y) => format!(
                "\\title{{{} ({}) -- {}}}\n\\author{{{}}}\n\\date{{}}",
                self.name, self.abbreviation, y, self.cirriculum.name,
            ),
            None => format!(
                "\\title{{{} ({})}}\n\\author{{{}}}\n\\date{{}}",
                self.name, self.abbreviation, self.cirriculum.name,
            ),
        };
        builder.add_to_preamble(title);

        // ── instructions ───────────────────────────────────────────────────
        let non_empty: Vec<&Instruction> = self
            .instructions
            .iter()
            .filter(|i| !i.is_empty())
            .collect();

        if !non_empty.is_empty() {
            let mut instr = String::from("\\section*{Instructions}\n\\begin{description}\n");
            for instruction in &non_empty {
                match instruction {
                    Instruction::Constants(entries) => {
                        instr.push_str("\\item[Constants] \\\\\n");
                        for (name, value) in entries {
                            instr.push_str(&format!("  ${}$ = {}\\\\\n", name, value));
                        }
                    }
                    Instruction::Formulae(formulae) => {
                        instr.push_str("\\item[Formulae] \\\\\n");
                        for formula in formulae {
                            instr.push_str(&format!("  ${}$\\\\\n", formula));
                        }
                    }
                    Instruction::RelativeAtomicMasses(entries) => {
                        instr.push_str("\\item[Relative Atomic Masses] \\\\\n");
                        for (symbol, mass) in entries {
                            instr.push_str(&format!("  $A_r({})$ = {}\\\\\n", symbol, mass));
                        }
                    }
                }
            }
            instr.push_str("\\end{description}\n");
            builder.add_content(instr);
        }

        // ── problems ───────────────────────────────────────────────────────
        builder.add_content("\\section*{Problems}".to_string());
        for (i, problem) in self.content.iter().enumerate() {
            let rendered = <ElementalQuestion as Renderer<Latex, Problem>>::render(problem)?;
            if i > 0 {
                builder.add_content("\\bigskip".to_string());
            }
            builder.add_content(rendered);
        }

        Ok(builder.build())
    }
}

impl Renderer<Html, Problem> for Paper {
    /// Renders the paper as a self-contained HTML fragment.
    ///
    /// Layout:
    /// 1. `<h1>` title with year (if present).
    /// 2. `<section class="instructions">` for non-empty instructions.
    /// 3. `<section class="problems">` with each problem in a
    ///    `<div class="problem">`.
    fn render(&self) -> anyhow::Result<String> {
        let mut out = String::new();

        // ── title ──────────────────────────────────────────────────────────
        let title = match self.year {
            Some(y) => format!(
                "<h1>{} ({}) &ndash; {}</h1>\n<p class=\"curriculum\">{}</p>\n",
                self.name, self.abbreviation, y, self.cirriculum.name,
            ),
            None => format!(
                "<h1>{} ({})</h1>\n<p class=\"curriculum\">{}</p>\n",
                self.name, self.abbreviation, self.cirriculum.name,
            ),
        };
        out.push_str(&title);

        // ── instructions ───────────────────────────────────────────────────
        let non_empty: Vec<&Instruction> = self
            .instructions
            .iter()
            .filter(|i| !i.is_empty())
            .collect();

        if !non_empty.is_empty() {
            out.push_str("<section class=\"instructions\">\n<h2>Instructions</h2>\n<dl>\n");
            for instruction in &non_empty {
                match instruction {
                    Instruction::Constants(entries) => {
                        out.push_str("<dt>Constants</dt>\n<dd>");
                        for (name, value) in entries {
                            out.push_str(&format!(
                                "<span class=\"constant\"><em>{}</em> = {}</span> ",
                                name, value
                            ));
                        }
                        out.push_str("</dd>\n");
                    }
                    Instruction::Formulae(formulae) => {
                        out.push_str("<dt>Formulae</dt>\n<dd>");
                        for formula in formulae {
                            out.push_str(&format!(
                                "<span class=\"formula\"><em>{}</em></span> ",
                                formula
                            ));
                        }
                        out.push_str("</dd>\n");
                    }
                    Instruction::RelativeAtomicMasses(entries) => {
                        out.push_str("<dt>Relative Atomic Masses</dt>\n<dd>");
                        for (symbol, mass) in entries {
                            out.push_str(&format!(
                                "<span class=\"ram\"><em>A<sub>r</sub>({})</em> = {}</span> ",
                                symbol, mass
                            ));
                        }
                        out.push_str("</dd>\n");
                    }
                }
            }
            out.push_str("</dl>\n</section>\n");
        }

        // ── problems ───────────────────────────────────────────────────────
        out.push_str("<section class=\"problems\">\n<h2>Problems</h2>\n");
        for (i, problem) in self.content.iter().enumerate() {
            let rendered = <ElementalQuestion as Renderer<Html, Problem>>::render(problem)?;
            out.push_str(&format!(
                "<div class=\"problem\" data-index=\"{}\">\n{}\n</div>\n",
                i + 1,
                rendered.trim()
            ));
        }
        out.push_str("</section>\n");

        Ok(out)
    }
}

impl Renderer<Markdown, Problem> for Paper {
    /// Renders the paper as Markdown.
    ///
    /// Layout:
    /// 1. `# Title` with year and curriculum.
    /// 2. `## Instructions` section.
    /// 3. `## Problems` with each problem preceded by its 1-based index as a
    ///    bold label.
    fn render(&self) -> anyhow::Result<String> {
        let mut out = String::new();

        // ── title ──────────────────────────────────────────────────────────
        let title = match self.year {
            Some(y) => format!("# {} ({}) – {}\n\n_{}_\n\n", self.name, self.abbreviation, y, self.cirriculum.name),
            None => format!("# {} ({})\n\n_{}_\n\n", self.name, self.abbreviation, self.cirriculum.name),
        };
        out.push_str(&title);

        // ── instructions ───────────────────────────────────────────────────
        let non_empty: Vec<&Instruction> = self
            .instructions
            .iter()
            .filter(|i| !i.is_empty())
            .collect();

        if !non_empty.is_empty() {
            out.push_str("## Instructions\n\n");
            for instruction in &non_empty {
                match instruction {
                    Instruction::Constants(entries) => {
                        out.push_str("**Constants:**\n\n");
                        for (name, value) in entries {
                            out.push_str(&format!("- *{}* = {}\n", name, value));
                        }
                        out.push('\n');
                    }
                    Instruction::Formulae(formulae) => {
                        out.push_str("**Formulae:**\n\n");
                        for formula in formulae {
                            out.push_str(&format!("- *{}*\n", formula));
                        }
                        out.push('\n');
                    }
                    Instruction::RelativeAtomicMasses(entries) => {
                        out.push_str("**Relative Atomic Masses:**\n\n");
                        for (symbol, mass) in entries {
                            out.push_str(&format!("- *A*_r_(*{}*) = {}\n", symbol, mass));
                        }
                        out.push('\n');
                    }
                }
            }
        }

        // ── problems ───────────────────────────────────────────────────────
        out.push_str("## Problems\n\n");
        for (i, problem) in self.content.iter().enumerate() {
            let rendered = <ElementalQuestion as Renderer<Markdown, Problem>>::render(problem)?;
            out.push_str(&format!("**{}.**\n\n{}\n\n", i + 1, rendered.trim()));
        }

        Ok(out)
    }
}