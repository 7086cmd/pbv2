use schema::db::{
    DbElementKind, DbElementRow, DbElementalProblemRow, DbElementalQuestionRow, DbListItemRow,
    DbListRow, DbParagraphElementRow, DbProblemCategoryRow, DbQuestionBlockKind, DbSingleProblemRow,
    DbTextRow,
};
use schema::{
    Element, ElementalQuestion, FontSize, Html, List, OrderFormat, OrderType, Paragraph, Problem,
    ProblemCategory, ProblemGroup, QuestionBlock, Renderer, SingleProblem, Text, TextAttributes,
};
use std::future::Future;
use std::pin::Pin;

// ── Shared DB helper ──────────────────────────────────────────────────────────

async fn db_pool() -> Result<sqlx::PgPool, String> {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/pbv2".to_string());
    sqlx::PgPool::connect(&url)
        .await
        .map_err(|e| format!("DB connection failed: {e}"))
}

// ── Category listing ──────────────────────────────────────────────────────────

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct CategoryItem {
    pub id: i64,
    pub curriculum_name: String,
    pub subject_name: String,
    pub subject_category: String,
    pub grade: i32,
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
    let pool = db_pool().await?;
    let rows = sqlx::query_as::<_, CategoryItem>(LIST_CATEGORIES_QUERY)
        .fetch_all(&pool)
        .await
        .map_err(|e| format!("DB query failed: {e}"))?;
    pool.close().await;
    Ok(rows)
}

// ── Problem listing ───────────────────────────────────────────────────────────

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct ProblemListItem {
    pub id: i64,
    pub preview: String,
}

const LIST_PROBLEMS_QUERY: &str = r#"
    SELECT
        sp.id,
        COALESCE(LEFT(convert_from(t.content, 'UTF-8'), 120), '(no preview)') AS preview
    FROM  single_problems sp
    JOIN  elemental_problems ep  ON ep.id = sp.problem_id
    JOIN  elemental_questions eq ON eq.id = ep.question_id
    JOIN  paragraph_elements pe  ON pe.paragraph_id = eq.content_id
                                 AND pe.position = 0
    JOIN  elements e             ON e.id = pe.element_id
    LEFT  JOIN texts t           ON t.id = e.text_id
    WHERE sp.category_id = $1
    ORDER BY sp.id
"#;

#[tauri::command]
pub async fn list_problems(category_id: i64) -> Result<Vec<ProblemListItem>, String> {
    let pool = db_pool().await?;
    let rows = sqlx::query_as::<_, ProblemListItem>(LIST_PROBLEMS_QUERY)
        .bind(category_id)
        .fetch_all(&pool)
        .await
        .map_err(|e| format!("list_problems query: {e}"))?;
    pool.close().await;
    Ok(rows)
}

// ── DB loader helpers ─────────────────────────────────────────────────────────

async fn load_text(pool: &sqlx::PgPool, text_id: i64) -> Result<Text, String> {
    let row: DbTextRow = sqlx::query_as(
        "SELECT id, formatting, content, font_size, color_r, color_g, color_b \
         FROM texts WHERE id = $1",
    )
    .bind(text_id)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("load_text({text_id}): {e}"))?;

    Ok(Text {
        formatting: row.formatting,
        content: row.content,
        attributes: TextAttributes {
            font_size: FontSize::from(row.font_size),
            color: (row.color_r as u8, row.color_g as u8, row.color_b as u8),
        },
    })
}

fn load_list<'a>(
    pool: &'a sqlx::PgPool,
    list_id: i64,
) -> Pin<Box<dyn Future<Output = Result<List, String>> + Send + 'a>> {
    Box::pin(async move {
        let list_row: DbListRow = sqlx::query_as(
            "SELECT id, order_type, order_format FROM lists WHERE id = $1",
        )
        .bind(list_id)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("load_list({list_id}): {e}"))?;

        let item_rows: Vec<DbListItemRow> = sqlx::query_as(
            "SELECT list_id, position, paragraph_id \
             FROM list_items WHERE list_id = $1 ORDER BY position",
        )
        .bind(list_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("load_list_items({list_id}): {e}"))?;

        let mut items = Vec::new();
        for item in item_rows {
            items.push(load_paragraph(pool, item.paragraph_id).await?);
        }

        Ok(List {
            items,
            order_type: OrderType::from(list_row.order_type),
            order_format: OrderFormat::from(list_row.order_format),
        })
    })
}

fn load_element<'a>(
    pool: &'a sqlx::PgPool,
    element_id: i64,
) -> Pin<Box<dyn Future<Output = Result<Element, String>> + Send + 'a>> {
    Box::pin(async move {
        let row: DbElementRow = sqlx::query_as(
            "SELECT id, kind, text_id, table_id, image_id, list_id, blank_id \
             FROM elements WHERE id = $1",
        )
        .bind(element_id)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("load_element({element_id}): {e}"))?;

        match row.kind {
            DbElementKind::Text => {
                let text_id = row.text_id.ok_or("missing text_id")?;
                let text = load_text(pool, text_id).await?;
                Ok(Element::Text(text))
            }
            DbElementKind::List => {
                let list_id = row.list_id.ok_or("missing list_id")?;
                let list = load_list(pool, list_id).await?;
                Ok(Element::List(list))
            }
            _ => {
                // Tables, images, blanks: render as placeholder
                let text = "(image)"
                    .parse::<Text>()
                    .map_err(|e| format!("placeholder text: {e}"))?;
                Ok(Element::Text(text))
            }
        }
    })
}

fn load_paragraph<'a>(
    pool: &'a sqlx::PgPool,
    para_id: i64,
) -> Pin<Box<dyn Future<Output = Result<Paragraph, String>> + Send + 'a>> {
    Box::pin(async move {
        let el_rows: Vec<DbParagraphElementRow> = sqlx::query_as(
            "SELECT paragraph_id, position, element_id \
             FROM paragraph_elements WHERE paragraph_id = $1 ORDER BY position",
        )
        .bind(para_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("load_paragraph({para_id}): {e}"))?;

        let mut elements = Vec::new();
        for el_row in el_rows {
            elements.push(load_element(pool, el_row.element_id).await?);
        }
        Ok(Paragraph::new(elements))
    })
}

async fn load_elemental_question(
    pool: &sqlx::PgPool,
    q_id: &str,
) -> Result<ElementalQuestion, String> {
    let row: DbElementalQuestionRow = sqlx::query_as(
        "SELECT id, content_id, answer_id, solution_id, choice_pool_id, \
                block_kind, block_lines, block_space \
         FROM elemental_questions WHERE id = $1",
    )
    .bind(q_id)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("load_elemental_question({q_id}): {e}"))?;

    let content = load_paragraph(pool, row.content_id).await?;

    let answer = if let Some(aid) = row.answer_id {
        Some(load_paragraph(pool, aid).await?)
    } else {
        None
    };

    let solution = if let Some(sid) = row.solution_id {
        Some(load_paragraph(pool, sid).await?)
    } else {
        None
    };

    let choice_pool = if let Some(cpid) = row.choice_pool_id {
        Some(load_list(pool, cpid).await?)
    } else {
        None
    };

    let block_type = match row.block_kind {
        DbQuestionBlockKind::Essay => QuestionBlock::Essay {
            lines: row.block_lines.unwrap_or(3) as usize,
        },
        DbQuestionBlockKind::Proof => {
            QuestionBlock::Proof { space: row.block_space.unwrap_or(4.0) }
        }
        DbQuestionBlockKind::Solve => {
            QuestionBlock::Solve { space: row.block_space.unwrap_or(4.0) }
        }
        DbQuestionBlockKind::None => QuestionBlock::None,
    };

    Ok(ElementalQuestion { id: row.id, content, answer, solution, choice_pool, block_type })
}

// ── Render DB problem ─────────────────────────────────────────────────────────

#[tauri::command]
pub async fn render_db_problem(id: i64) -> Result<String, String> {
    let pool = db_pool().await?;

    let sp_row: DbSingleProblemRow = sqlx::query_as(
        "SELECT id, problem_id, category_id FROM single_problems WHERE id = $1",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| format!("single_problems({id}): {e}"))?;

    let ep_row: DbElementalProblemRow = sqlx::query_as(
        "SELECT id, kind, question_id FROM elemental_problems WHERE id = $1",
    )
    .bind(sp_row.problem_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| format!("elemental_problems: {e}"))?;

    let q_id = ep_row.question_id.ok_or("no question_id on elemental_problem")?;
    let eq = load_elemental_question(&pool, &q_id).await?;

    let cat_row: DbProblemCategoryRow = sqlx::query_as(
        "SELECT id, cirriculum, subject, grade, categories, origin \
         FROM problem_categories WHERE id = $1",
    )
    .bind(sp_row.category_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| format!("problem_categories: {e}"))?;

    let category = ProblemCategory {
        cirriculum: cat_row.cirriculum,
        subject: cat_row.subject,
        grade: cat_row.grade,
        categories: cat_row.categories,
        origin: cat_row.origin,
    };

    pool.close().await;

    let sp = SingleProblem { problem: eq, category };
    <SingleProblem as Renderer<Html, Problem>>::render(&sp).map_err(|e| e.to_string())
}

// ── Demo renderers (hard-coded) ───────────────────────────────────────────────

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
            simple_question(
                "q1",
                "What is the primary product of cellular respiration?",
                QuestionBlock::Essay { lines: 3 },
            ),
            ElementalQuestion {
                id: "q2".to_owned(),
                content: para("Answer all sub-questions."),
                answer: None,
                solution: None,
                choice_pool: Some(List {
                    items: vec![
                        para("Name the two stages of cellular respiration."),
                        para("Where does glycolysis occur in the cell?"),
                    ],
                    order_type: OrderType::LowercaseAlphabetic,
                    order_format: OrderFormat::Parenthesis,
                }),
                block_type: QuestionBlock::None,
            },
        ],
        category: category(),
    };
    <ProblemGroup as Renderer<Html, Problem>>::render(&pg).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn render_single_problem() -> Result<String, String> {
    let sp = SingleProblem {
        problem: simple_question(
            "q3",
            "Describe the process of photosynthesis, including the reactants and products.",
            QuestionBlock::Solve { space: 8.0 },
        ),
        category: category(),
    };
    <SingleProblem as Renderer<Html, Problem>>::render(&sp).map_err(|e| e.to_string())
}
