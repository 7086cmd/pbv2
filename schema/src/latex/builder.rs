pub struct LatexBuilder {
    document_class: DocumentClass,
    preamble: Vec<String>,
    content: Vec<String>,
}

pub enum DocumentClass {
    Article { toc: bool },
    Report,
    Book,
    Standalone,
    Subfile { parent: String },
}

impl ToString for DocumentClass {
    fn to_string(&self) -> String {
        match self {
            DocumentClass::Article { toc: _ } => "\\documentclass{article}".to_string(),
            DocumentClass::Report => "\\documentclass{report}".to_string(),
            DocumentClass::Book => "\\documentclass{book}".to_string(),
            DocumentClass::Standalone => "\\documentclass{standalone}".to_string(),
            DocumentClass::Subfile { parent } => format!("\\documentclass[{parent}]{{subfile}}"),
        }
    }
}

impl DocumentClass {
    pub fn require_toc(&self) -> bool {
        matches!(
            self,
            DocumentClass::Article { toc: true } | DocumentClass::Report | DocumentClass::Book
        )
    }

    pub fn require_title(&self) -> bool {
        !matches!(
            self,
            DocumentClass::Standalone | DocumentClass::Subfile { .. }
        )
    }

    pub fn require_extra_preamble(&self) -> bool {
        !matches!(self, DocumentClass::Standalone)
    }
}

impl LatexBuilder {
    pub fn new(document_class: DocumentClass) -> Self {
        let mut ego = Self {
            document_class,
            preamble: Vec::new(),
            content: Vec::new(),
        };

        ego.add_to_preamble(include_str!("../preambles/preamble.tex").to_string());

        if ego.document_class.require_extra_preamble() {
            ego.add_to_preamble(include_str!("../preambles/document.tex").to_string());
        }

        ego
    }

    pub fn add_to_preamble(&mut self, line: String) {
        self.preamble.extend(line.lines().map(|s| s.to_string()));
    }

    pub fn add_content(&mut self, line: String) {
        self.content.extend(line.lines().map(|s| s.to_string()));
    }

    pub fn build(&mut self) -> String {
        let mut latex = String::new();
        latex.push_str(&self.document_class.to_string());
        latex.push_str("\n");

        for line in &self.preamble {
            latex.push_str(line);
            latex.push_str("\n");
        }

        if self.document_class.require_title() {
            latex.push_str("\\title{Title}\n\\author{Author}\n\\date{\\today}\n\n");
        }

        latex.push_str("\n\\begin{document}\n\n");

        if self.document_class.require_title() {
            latex.push_str("\\maketitle\n\n");
        }

        if self.document_class.require_toc() {
            latex.push_str("\\tableofcontents\n\n");
        }

        for line in &self.content {
            latex.push_str(line);
            latex.push_str("\n");
        }

        latex.push_str("\\end{document}\n");
        latex
    }
}
