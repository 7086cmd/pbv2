use schema::{
    Element, ElementalProblem, ElementalQuestion, Html, OrderFormat, OrderType, Paragraph,
    Problem, ProblemCategory, ProblemGroup, QuestionBlock, QuestionSeries, Renderer,
    SingleProblem, Text,
};

// ── Category listing ───────────────────────────────────────────────────────────

/// Flattened view of one `problem_categories` row joined with its curriculum,
/// subject, and a count of associated `single_problems`.
#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct CategoryItem {
    pub id: i64,
    pub curriculum_name: String,
    pub subject_name: String,
    /// Postgres `subject_category` enum cast to text.
    pub subject_category: String,
    pub grade: i32,
    /// Free-form taxonomy tags stored as `text[]`.
    pub categories: Vec<String>,
    pub origin: Option<i32>,
    pub problem_count: i64,
}

const LIST_CATEGORIES_QUERY: &str = r#"
    SELECT
        pc.id,
        COALESCE(cu.name, '')           AS curriculum_name,
        COALESCE(su.name, '')           AS subject_name,
        COALESCE(su.category::text, '') AS subject_category,
        pc.grade,
        pc.categories,
        pc.origin,
        COUNT(sp.id)                    AS problem_count
    FROM  problem_categories pc
    LEFT  JOIN curriculums   cu ON cu.id = pc.cirriculum
    LEFT  JOIN subjects      su ON su.id = pc.subject
    LEFT  JOIN single_problems sp ON sp.category_id = pc.id
    GROUP BY pc.id, cu.name, su.name, su.category
    ORDER BY cu.name, su.category, pc.grade, su.name
"#;

#[tauri::command]
pub async fn list_categories() -> Result<Vec<CategoryItem>, String> {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/pbv2".to_string());

    let pool = sqlx::PgPool::connect(&url)
        .await
        .map_err(|e| format!("DB connection failed: {e}"))?;

    let rows = sqlx::query_as::<_, CategoryItem>(LIST_CATEGORIES_QUERY)
        .fetch_all(&pool)
        .await
        .map_err(|e| format!("DB query failed: {e}"))?;

    pool.close().await;
    Ok(rows)
}

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
