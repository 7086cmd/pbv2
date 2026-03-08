use schema::{
    Element, ElementalProblem, ElementalQuestion, Html, OrderFormat, OrderType, Paragraph,
    Problem, ProblemCategory, ProblemGroup, QuestionBlock, QuestionSeries, Renderer,
    SingleProblem, Text,
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

fn simple_question(id: &str, text: &str, block_type: QuestionBlock) -> ElementalQuestion {
    ElementalQuestion {
        id: id.to_owned(),
        content: para(text),
        answer: None,
        solution: None,
        choice_pool: None,
        block_type,
    }
}

#[tauri::command]
pub fn render_problem_group() -> Result<String, String> {
    let pg = ProblemGroup {
        material: para(
            "Read the following passage about cellular respiration and answer the questions below.",
        ),
        problems: vec![
            ElementalProblem::Question(simple_question(
                "q1",
                "What is the primary product of cellular respiration?",
                QuestionBlock::Essay { lines: 3 },
            )),
            ElementalProblem::Block(QuestionSeries {
                content: para("Answer all sub-questions."),
                questions: vec![
                    simple_question(
                        "q2a",
                        "Name the two stages of cellular respiration.",
                        QuestionBlock::Essay { lines: 2 },
                    ),
                    simple_question(
                        "q2b",
                        "Where does glycolysis occur in the cell?",
                        QuestionBlock::Essay { lines: 2 },
                    ),
                ],
                order_type: OrderType::LowercaseAlphabetic,
                order_format: OrderFormat::Parenthesis,
                order_resume: false,
            }),
        ],
        category: category(),
    };
    <ProblemGroup as Renderer<Html, Problem>>::render(&pg).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn render_single_problem() -> Result<String, String> {
    let sp = SingleProblem {
        problem: ElementalProblem::Question(simple_question(
            "q3",
            "Describe the process of photosynthesis, including the reactants and products.",
            QuestionBlock::Solve { space: 8.0 },
        )),
        category: category(),
    };
    <SingleProblem as Renderer<Html, Problem>>::render(&sp).map_err(|e| e.to_string())
}
