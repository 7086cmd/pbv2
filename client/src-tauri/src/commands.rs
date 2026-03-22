use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use schema::db::{
    DbElementKind, DbElementRow, DbElementalProblemRow, DbElementalQuestionRow, DbImageFormat,
    DbImageKind, DbImageRow, DbListItemRow, DbListRow, DbOrderFormat, DbOrderType,
    DbParagraphElementRow, DbProblemCategoryRow, DbQuestionBlockKind, DbSingleProblemRow,
    DbTextRow,
};
use schema::{
    BinaryImage, CompiledGraph, Element, ElementalQuestion, FontSize, Html, Image, ImageFormat,
    List, OrderFormat, OrderType, Paragraph, PdfSvgImage, Problem, ProblemCategory, ProblemGroup,
    QuestionBlock, Renderer, SingleProblem, Text, TextAttributes, TextFlags, TextFormat,
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
            DbElementKind::Image => {
                let image_id = row.image_id.ok_or("missing image_id")?;
                let img_row: DbImageRow = sqlx::query_as(
                    "SELECT id, kind, buffer, format, filename, url, tex_code, png_content, \
                     svg_content, width_ratio, pdf_buffer FROM images WHERE id = $1",
                )
                .bind(image_id)
                .fetch_one(pool)
                .await
                .map_err(|e| format!("load image row({image_id}): {e}"))?;

                let image = match img_row.kind {
                    DbImageKind::Binary => {
                        let buf = img_row.buffer.unwrap_or_default();
                        let format = match img_row.format {
                            Some(DbImageFormat::Jpeg) => ImageFormat::Jpeg,
                            _ => ImageFormat::Png,
                        };
                        Image::Binary(BinaryImage {
                            buffer: buf,
                            format,
                            filename: img_row.filename,
                            width_ratio: img_row.width_ratio,
                        })
                    }
                    DbImageKind::Url => {
                        Image::Url(img_row.url.unwrap_or_default())
                    }
                    DbImageKind::Latex => {
                        let tex_code = img_row.tex_code.unwrap_or_default();
                        let png_content = img_row.png_content.unwrap_or_default();
                        // svg_content not stored in Latex rows; use empty string
                        Image::Latex(CompiledGraph::new(tex_code, String::new(), png_content))
                    }
                    DbImageKind::PdfSvg => {
                        let pdf_buffer = img_row.pdf_buffer.unwrap_or_default();
                        let svg_content = img_row.svg_content.unwrap_or_default();
                        Image::PdfSvg(PdfSvgImage {
                            pdf_buffer,
                            svg_content,
                            width_ratio: img_row.width_ratio,
                        })
                    }
                };
                Ok(Element::Image(image))
            }
            _ => {
                // Tables, blanks: render as placeholder
                let text = "(unsupported element)"
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

// ── Editor DTOs ───────────────────────────────────────────────────────────────

/// A serialisable segment of a problem's content paragraph.
/// Used to bridge the Vue rich editor and the normalised PostgreSQL schema.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum ContentSegmentDto {
    Text {
        text: String,
        bold: bool,
        italic: bool,
        underline: bool,
        underwave: bool,
        strikethrough: bool,
        superscript: bool,
        subscript: bool,
        monospace: bool,
        formula: bool,
        red: bool,
    },
    Image {
        url: Option<String>,
        data_uri: Option<String>,
        width_ratio: Option<f64>,
    },
    List {
        order_type: String,
        order_format: String,
        /// Each item is a flat sequence of Text/Image segments.
        items: Vec<Vec<ContentSegmentDto>>,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChoiceDto {
    pub content: Vec<ContentSegmentDto>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChoicePoolDto {
    /// One of: "uppercase_alphabetic" | "lowercase_alphabetic" | "uppercase_roman" |
    ///         "lowercase_roman" | "decimal" | "unordered"
    pub order_type: String,
    /// One of: "period" | "parenthesis" | "right_parenthesis" | "none"
    pub order_format: String,
    pub choices: Vec<ChoiceDto>,
}

#[derive(Debug, serde::Serialize)]
pub struct ProblemContentResponse {
    pub single_problem_id: i64,
    pub question_id: String,
    pub content: Vec<ContentSegmentDto>,
    pub choice_pool: Option<ChoicePoolDto>,
}

// ── Editor helpers ────────────────────────────────────────────────────────────

fn text_to_dtos(text: &Text) -> Vec<ContentSegmentDto> {
    let n = text.formatting.len() / 8;
    if n == 0 {
        let s = String::from_utf8_lossy(&text.content).to_string();
        return vec![ContentSegmentDto::Text {
            text: s,
            bold: false,
            italic: false,
            underline: false,
            underwave: false,
            strikethrough: false,
            superscript: false,
            subscript: false,
            monospace: false,
            formula: false,
            red: false,
        }];
    }
    (0..n)
        .map(|i| {
            let fmt = TextFormat::from_bytes(&text.formatting[i * 8..(i + 1) * 8]);
            let start = fmt.start as usize;
            let end = if i + 1 < n {
                TextFormat::from_bytes(&text.formatting[(i + 1) * 8..(i + 2) * 8]).start as usize
            } else {
                text.content.len()
            };
            let run = String::from_utf8_lossy(&text.content[start..end]).to_string();
            let f = fmt.flags;
            ContentSegmentDto::Text {
                text: run,
                bold: f.contains(TextFlags::BOLD),
                italic: f.contains(TextFlags::ITALIC),
                underline: f.contains(TextFlags::UNDERLINE),
                underwave: f.contains(TextFlags::UNDERWAVE),
                strikethrough: f.contains(TextFlags::STRIKETHROUGH),
                superscript: f.contains(TextFlags::SUPERSCRIPT),
                subscript: f.contains(TextFlags::SUBSCRIPT),
                monospace: f.contains(TextFlags::MONOSPACE),
                formula: f.contains(TextFlags::FORMULA),
                red: f.contains(TextFlags::RED),
            }
        })
        .collect()
}

async fn load_image_dto(pool: &sqlx::PgPool, image_id: i64) -> Result<ContentSegmentDto, String> {
    let row: DbImageRow = sqlx::query_as(
        "SELECT id, kind, buffer, format, filename, url, tex_code, png_content, \
         svg_content, width_ratio, pdf_buffer FROM images WHERE id = $1",
    )
    .bind(image_id)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("load_image_dto({image_id}): {e}"))?;

    let dto = match row.kind {
        DbImageKind::Binary => {
            let buf = row.buffer.unwrap_or_default();
            let mime = match row.format {
                Some(DbImageFormat::Jpeg) => "jpeg",
                _ => "png",
            };
            let data_uri = format!("data:image/{mime};base64,{}", BASE64.encode(&buf));
            ContentSegmentDto::Image {
                url: None,
                data_uri: Some(data_uri),
                width_ratio: row.width_ratio,
            }
        }
        DbImageKind::Url => ContentSegmentDto::Image {
            url: row.url,
            data_uri: None,
            width_ratio: None,
        },
        DbImageKind::Latex => {
            let buf = row.png_content.unwrap_or_default();
            let data_uri = format!("data:image/png;base64,{}", BASE64.encode(&buf));
            ContentSegmentDto::Image {
                url: None,
                data_uri: Some(data_uri),
                width_ratio: row.width_ratio,
            }
        }
        DbImageKind::PdfSvg => {
            let svg = row.svg_content.unwrap_or_default();
            let data_uri =
                format!("data:image/svg+xml;base64,{}", BASE64.encode(svg.as_bytes()));
            ContentSegmentDto::Image {
                url: None,
                data_uri: Some(data_uri),
                width_ratio: row.width_ratio,
            }
        }
    };
    Ok(dto)
}

fn load_paragraph_segments<'a>(
    pool: &'a sqlx::PgPool,
    para_id: i64,
) -> Pin<Box<dyn Future<Output = Result<Vec<ContentSegmentDto>, String>> + Send + 'a>> {
    Box::pin(async move {
        let el_rows: Vec<DbParagraphElementRow> = sqlx::query_as(
            "SELECT paragraph_id, position, element_id \
             FROM paragraph_elements WHERE paragraph_id = $1 ORDER BY position",
        )
        .bind(para_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("paragraph_elements({para_id}): {e}"))?;

        let mut segments = Vec::new();
        for el_row in el_rows {
            let el: DbElementRow = sqlx::query_as(
                "SELECT id, kind, text_id, table_id, image_id, list_id, blank_id \
                 FROM elements WHERE id = $1",
            )
            .bind(el_row.element_id)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("elements({}): {e}", el_row.element_id))?;

            match el.kind {
                DbElementKind::Text => {
                    if let Some(text_id) = el.text_id {
                        let text = load_text(pool, text_id).await?;
                        segments.extend(text_to_dtos(&text));
                    }
                }
                DbElementKind::Image => {
                    if let Some(image_id) = el.image_id {
                        segments.push(load_image_dto(pool, image_id).await?);
                    }
                }
                DbElementKind::List => {
                    if let Some(lid) = el.list_id {
                        let list_row: DbListRow = sqlx::query_as(
                            "SELECT id, order_type, order_format FROM lists WHERE id = $1",
                        )
                        .bind(lid)
                        .fetch_one(pool)
                        .await
                        .map_err(|e| format!("lists({lid}): {e}"))?;

                        let item_rows: Vec<DbListItemRow> = sqlx::query_as(
                            "SELECT list_id, position, paragraph_id \
                             FROM list_items WHERE list_id = $1 ORDER BY position",
                        )
                        .bind(lid)
                        .fetch_all(pool)
                        .await
                        .map_err(|e| format!("list_items({lid}): {e}"))?;

                        let mut items = Vec::new();
                        for item in item_rows {
                            items.push(load_paragraph_segments(pool, item.paragraph_id).await?);
                        }

                        let order_type = match list_row.order_type {
                            DbOrderType::UppercaseAlphabetic => "uppercase_alphabetic",
                            DbOrderType::LowercaseAlphabetic => "lowercase_alphabetic",
                            DbOrderType::UppercaseRoman => "uppercase_roman",
                            DbOrderType::LowercaseRoman => "lowercase_roman",
                            DbOrderType::Decimal => "decimal",
                            DbOrderType::Unordered => "unordered",
                        }
                        .to_owned();

                        let order_format = match list_row.order_format {
                            DbOrderFormat::Period => "period",
                            DbOrderFormat::Parenthesis => "parenthesis",
                            DbOrderFormat::RightParenthesis => "right_parenthesis",
                            DbOrderFormat::None => "none",
                        }
                        .to_owned();

                        segments.push(ContentSegmentDto::List { order_type, order_format, items });
                    }
                }
                _ => {}
            }
        }
        Ok(segments)
    })
}

async fn load_choice_pool_dto(
    pool: &sqlx::PgPool,
    list_id: i64,
) -> Result<ChoicePoolDto, String> {
    let list_row: DbListRow =
        sqlx::query_as("SELECT id, order_type, order_format FROM lists WHERE id = $1")
            .bind(list_id)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("load_choice_pool({list_id}): {e}"))?;

    let item_rows: Vec<DbListItemRow> = sqlx::query_as(
        "SELECT list_id, position, paragraph_id \
         FROM list_items WHERE list_id = $1 ORDER BY position",
    )
    .bind(list_id)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("load_choice_items({list_id}): {e}"))?;

    let mut choices = Vec::new();
    for item in item_rows {
        let content = load_paragraph_segments(pool, item.paragraph_id).await?;
        choices.push(ChoiceDto { content });
    }

    let order_type = match list_row.order_type {
        DbOrderType::UppercaseAlphabetic => "uppercase_alphabetic",
        DbOrderType::LowercaseAlphabetic => "lowercase_alphabetic",
        DbOrderType::UppercaseRoman => "uppercase_roman",
        DbOrderType::LowercaseRoman => "lowercase_roman",
        DbOrderType::Decimal => "decimal",
        DbOrderType::Unordered => "unordered",
    }
    .to_owned();

    let order_format = match list_row.order_format {
        DbOrderFormat::Period => "period",
        DbOrderFormat::Parenthesis => "parenthesis",
        DbOrderFormat::RightParenthesis => "right_parenthesis",
        DbOrderFormat::None => "none",
    }
    .to_owned();

    Ok(ChoicePoolDto { order_type, order_format, choices })
}

fn segment_to_text(seg: &ContentSegmentDto) -> Option<Text> {
    if let ContentSegmentDto::Text {
        text,
        bold,
        italic,
        underline,
        underwave,
        strikethrough,
        superscript,
        subscript,
        monospace,
        formula,
        red,
    } = seg
    {
        let mut flags = TextFlags::empty();
        if *bold { flags |= TextFlags::BOLD; }
        if *italic { flags |= TextFlags::ITALIC; }
        if *underline { flags |= TextFlags::UNDERLINE; }
        if *underwave { flags |= TextFlags::UNDERWAVE; }
        if *strikethrough { flags |= TextFlags::STRIKETHROUGH; }
        if *superscript { flags |= TextFlags::SUPERSCRIPT; }
        if *subscript { flags |= TextFlags::SUBSCRIPT; }
        if *monospace { flags |= TextFlags::MONOSPACE; }
        if *formula { flags |= TextFlags::FORMULA; }
        if *red { flags |= TextFlags::RED; }
        let fmt = TextFormat { language: 0, start: 0, flags };
        Some(Text {
            formatting: fmt.to_bytes().to_vec(),
            content: text.as_bytes().to_vec(),
            attributes: TextAttributes::default(),
        })
    } else {
        None
    }
}

// ── get_problem_content ───────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_problem_content(id: i64) -> Result<ProblemContentResponse, String> {
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

    let q_id = ep_row.question_id.ok_or("no question_id")?;

    let content_id: i64 = sqlx::query_scalar(
        "SELECT content_id FROM elemental_questions WHERE id = $1",
    )
    .bind(&q_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| format!("elemental_questions content_id: {e}"))?;

    let choice_pool_id: Option<i64> = sqlx::query_scalar(
        "SELECT choice_pool_id FROM elemental_questions WHERE id = $1",
    )
    .bind(&q_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| format!("elemental_questions choice_pool_id: {e}"))?;

    let content = load_paragraph_segments(&pool, content_id).await?;

    let choice_pool = if let Some(cpid) = choice_pool_id {
        Some(load_choice_pool_dto(&pool, cpid).await?)
    } else {
        None
    };

    pool.close().await;
    Ok(ProblemContentResponse {
        single_problem_id: id,
        question_id: q_id,
        content,
        choice_pool,
    })
}

// ── Insertion helper ──────────────────────────────────────────────────────────

/// Insert a single Text or Image segment as an element in a paragraph.
/// List segments (and unknown kinds) are silently skipped at the item level.
async fn insert_segment_as_element(
    conn: &mut sqlx::PgConnection,
    para_id: i64,
    pos: i32,
    seg: &ContentSegmentDto,
) -> Result<(), String> {
    let el_id: i64 = match seg {
        ContentSegmentDto::Text { .. } => {
            let text = segment_to_text(seg).unwrap();
            let text_id: i64 = sqlx::query_scalar(
                "INSERT INTO texts (formatting, content, font_size, color_r, color_g, color_b) \
                 VALUES ($1, $2, 'normal', 0, 0, 0) RETURNING id",
            )
            .bind(&text.formatting)
            .bind(&text.content)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| format!("insert text: {e}"))?;

            sqlx::query_scalar(
                "INSERT INTO elements (kind, text_id) VALUES ('text', $1) RETURNING id",
            )
            .bind(text_id)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| format!("insert element(text): {e}"))?
        }
        ContentSegmentDto::Image { url, data_uri, width_ratio } => {
            let image_id: i64 = if let Some(uri) = data_uri {
                let (mime, b64) = uri
                    .strip_prefix("data:")
                    .and_then(|s| s.split_once(';'))
                    .and_then(|(mime, rest)| rest.strip_prefix("base64,").map(|b64| (mime, b64)))
                    .ok_or("invalid data URI")?;
                let buf = BASE64.decode(b64).map_err(|e| format!("base64 decode: {e}"))?;
                let fmt_str = if mime.contains("jpeg") { "jpeg" } else { "png" };
                sqlx::query_scalar(
                    "INSERT INTO images (kind, buffer, format, width_ratio) \
                     VALUES ('binary', $1, $2::image_format, $3) RETURNING id",
                )
                .bind(&buf)
                .bind(fmt_str)
                .bind(width_ratio)
                .fetch_one(&mut *conn)
                .await
                .map_err(|e| format!("insert binary image: {e}"))?
            } else if let Some(u) = url {
                sqlx::query_scalar(
                    "INSERT INTO images (kind, url) VALUES ('url', $1) RETURNING id",
                )
                .bind(u)
                .fetch_one(&mut *conn)
                .await
                .map_err(|e| format!("insert url image: {e}"))?
            } else {
                return Err("image has neither url nor data_uri".to_owned());
            };

            sqlx::query_scalar(
                "INSERT INTO elements (kind, image_id) VALUES ('image', $1) RETURNING id",
            )
            .bind(image_id)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| format!("insert element(image): {e}"))?
        }
        ContentSegmentDto::List { .. } => return Ok(()), // nested lists not supported at item level
    };

    sqlx::query(
        "INSERT INTO paragraph_elements (paragraph_id, position, element_id) \
         VALUES ($1, $2, $3)",
    )
    .bind(para_id)
    .bind(pos)
    .bind(el_id)
    .execute(&mut *conn)
    .await
    .map_err(|e| format!("insert paragraph_element: {e}"))?;

    Ok(())
}

// ── save_problem_content ──────────────────────────────────────────────────────

#[tauri::command]
pub async fn save_problem_content(
    id: i64,
    segments: Vec<ContentSegmentDto>,
    choice_pool: Option<ChoicePoolDto>,
) -> Result<(), String> {
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

    let q_id = ep_row.question_id.ok_or("no question_id")?;
    let content_id: i64 = sqlx::query_scalar(
        "SELECT content_id FROM elemental_questions WHERE id = $1",
    )
    .bind(&q_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| format!("elemental_questions content_id: {e}"))?;

    let old_choice_pool_id: Option<i64> = sqlx::query_scalar(
        "SELECT choice_pool_id FROM elemental_questions WHERE id = $1",
    )
    .bind(&q_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| format!("elemental_questions choice_pool_id: {e}"))?;

    let mut tx = pool.begin().await.map_err(|e| format!("begin tx: {e}"))?;

    let old_el_ids: Vec<i64> = sqlx::query_scalar(
        "SELECT element_id FROM paragraph_elements WHERE paragraph_id = $1",
    )
    .bind(content_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| format!("fetch old elements: {e}"))?;

    sqlx::query("DELETE FROM paragraph_elements WHERE paragraph_id = $1")
        .bind(content_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("delete paragraph_elements: {e}"))?;

    for el_id in old_el_ids {
        let el: DbElementRow = sqlx::query_as(
            "SELECT id, kind, text_id, table_id, image_id, list_id, blank_id \
             FROM elements WHERE id = $1",
        )
        .bind(el_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| format!("fetch element {el_id}: {e}"))?
        .ok_or_else(|| format!("element {el_id} not found"))?;

        sqlx::query("DELETE FROM elements WHERE id = $1")
            .bind(el_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("delete element {el_id}: {e}"))?;

        if el.kind == DbElementKind::Text {
            if let Some(text_id) = el.text_id {
                sqlx::query("DELETE FROM texts WHERE id = $1")
                    .bind(text_id)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| format!("delete text {text_id}: {e}"))?;
            }
        } else if el.kind == DbElementKind::Image {
            if let Some(image_id) = el.image_id {
                sqlx::query("DELETE FROM images WHERE id = $1")
                    .bind(image_id)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| format!("delete image {image_id}: {e}"))?;
            }
        } else if el.kind == DbElementKind::List {
            if let Some(lid) = el.list_id {
                let para_ids: Vec<i64> = sqlx::query_scalar(
                    "SELECT paragraph_id FROM list_items WHERE list_id = $1",
                )
                .bind(lid)
                .fetch_all(&mut *tx)
                .await
                .map_err(|e| format!("fetch list item paragraphs({lid}): {e}"))?;

                // Delete list row — cascades list_items
                sqlx::query("DELETE FROM lists WHERE id = $1")
                    .bind(lid)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| format!("delete list {lid}: {e}"))?;

                for para_id in para_ids {
                    let nested_els: Vec<DbElementRow> = sqlx::query_as(
                        "SELECT e.id, e.kind, e.text_id, e.table_id, e.image_id, \
                                e.list_id, e.blank_id \
                         FROM paragraph_elements pe \
                         JOIN elements e ON e.id = pe.element_id \
                         WHERE pe.paragraph_id = $1",
                    )
                    .bind(para_id)
                    .fetch_all(&mut *tx)
                    .await
                    .map_err(|e| format!("fetch nested elements({para_id}): {e}"))?;

                    // Delete item paragraph — cascades paragraph_elements
                    sqlx::query("DELETE FROM paragraphs WHERE id = $1")
                        .bind(para_id)
                        .execute(&mut *tx)
                        .await
                        .map_err(|e| format!("delete list item paragraph {para_id}: {e}"))?;

                    for nel in nested_els {
                        sqlx::query("DELETE FROM elements WHERE id = $1")
                            .bind(nel.id)
                            .execute(&mut *tx)
                            .await
                            .map_err(|e| format!("delete nested element {}: {e}", nel.id))?;
                        if nel.kind == DbElementKind::Text {
                            if let Some(tid) = nel.text_id {
                                sqlx::query("DELETE FROM texts WHERE id = $1")
                                    .bind(tid)
                                    .execute(&mut *tx)
                                    .await
                                    .map_err(|e| format!("delete nested text {tid}: {e}"))?;
                            }
                        } else if nel.kind == DbElementKind::Image {
                            if let Some(iid) = nel.image_id {
                                sqlx::query("DELETE FROM images WHERE id = $1")
                                    .bind(iid)
                                    .execute(&mut *tx)
                                    .await
                                    .map_err(|e| format!("delete nested image {iid}: {e}"))?;
                            }
                        }
                    }
                }
            }
        }
    }

    for (pos, seg) in segments.iter().enumerate() {
        let pos = pos as i32;
        match seg {
            ContentSegmentDto::Text { .. } | ContentSegmentDto::Image { .. } => {
                insert_segment_as_element(&mut *tx, content_id, pos, seg).await?;
            }
            ContentSegmentDto::List { order_type, order_format, items } => {
                let new_list_id: i64 = sqlx::query_scalar(
                    "INSERT INTO lists (order_type, order_format) \
                     VALUES ($1::order_type, $2::order_format) RETURNING id",
                )
                .bind(order_type)
                .bind(order_format)
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| format!("insert list: {e}"))?;

                for (item_pos, item_segs) in items.iter().enumerate() {
                    let item_para_id: i64 =
                        sqlx::query_scalar("INSERT INTO paragraphs DEFAULT VALUES RETURNING id")
                            .fetch_one(&mut *tx)
                            .await
                            .map_err(|e| format!("insert list item paragraph: {e}"))?;

                    sqlx::query(
                        "INSERT INTO list_items (list_id, position, paragraph_id) \
                         VALUES ($1, $2, $3)",
                    )
                    .bind(new_list_id)
                    .bind(item_pos as i32)
                    .bind(item_para_id)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| format!("insert list_item: {e}"))?;

                    for (seg_pos, item_seg) in item_segs.iter().enumerate() {
                        insert_segment_as_element(
                            &mut *tx,
                            item_para_id,
                            seg_pos as i32,
                            item_seg,
                        )
                        .await?;
                    }
                }

                let el_id: i64 = sqlx::query_scalar(
                    "INSERT INTO elements (kind, list_id) VALUES ('list', $1) RETURNING id",
                )
                .bind(new_list_id)
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| format!("insert element(list): {e}"))?;

                sqlx::query(
                    "INSERT INTO paragraph_elements (paragraph_id, position, element_id) \
                     VALUES ($1, $2, $3)",
                )
                .bind(content_id)
                .bind(pos)
                .bind(el_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("insert paragraph_element(list): {e}"))?;
            }
        }
    }

    // ── Save choice pool ──────────────────────────────────────────────────────

    // Null out old FK so we can freely delete the old list
    if old_choice_pool_id.is_some() {
        sqlx::query("UPDATE elemental_questions SET choice_pool_id = NULL WHERE id = $1")
            .bind(&q_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("null choice_pool_id: {e}"))?;
    }

    // Delete old choice pool (list, its item paragraphs, and their elements)
    if let Some(old_cpid) = old_choice_pool_id {
        let old_para_ids: Vec<i64> =
            sqlx::query_scalar("SELECT paragraph_id FROM list_items WHERE list_id = $1")
                .bind(old_cpid)
                .fetch_all(&mut *tx)
                .await
                .map_err(|e| format!("fetch old choice paragraphs: {e}"))?;

        sqlx::query("DELETE FROM lists WHERE id = $1")
            .bind(old_cpid)
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("delete old list: {e}"))?;

        for para_id in old_para_ids {
            // Collect element metadata before touching the paragraph
            let els: Vec<DbElementRow> = sqlx::query_as(
                "SELECT e.id, e.kind, e.text_id, e.table_id, e.image_id, e.list_id, e.blank_id \
                 FROM paragraph_elements pe \
                 JOIN elements e ON e.id = pe.element_id \
                 WHERE pe.paragraph_id = $1",
            )
            .bind(para_id)
            .fetch_all(&mut *tx)
            .await
            .map_err(|e| format!("fetch choice elements for para {para_id}: {e}"))?;

            // Delete paragraph first — cascades paragraph_elements, freeing the
            // paragraph_elements_element_id_fkey constraint before we touch elements
            sqlx::query("DELETE FROM paragraphs WHERE id = $1")
                .bind(para_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("delete choice paragraph {para_id}: {e}"))?;

            // Now delete each element (no FK from paragraph_elements anymore)
            for el in els {
                sqlx::query("DELETE FROM elements WHERE id = $1")
                    .bind(el.id)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| format!("delete choice element {}: {e}", el.id))?;

                if el.kind == DbElementKind::Text {
                    if let Some(tid) = el.text_id {
                        sqlx::query("DELETE FROM texts WHERE id = $1")
                            .bind(tid)
                            .execute(&mut *tx)
                            .await
                            .map_err(|e| format!("delete choice text {tid}: {e}"))?;
                    }
                } else if el.kind == DbElementKind::Image {
                    if let Some(iid) = el.image_id {
                        sqlx::query("DELETE FROM images WHERE id = $1")
                            .bind(iid)
                            .execute(&mut *tx)
                            .await
                            .map_err(|e| format!("delete choice image {iid}: {e}"))?;
                    }
                }
            }
        }
    }

    // Create new choice pool
    if let Some(pool_dto) = choice_pool {
        if !pool_dto.choices.is_empty() {
            let new_list_id: i64 = sqlx::query_scalar(
                "INSERT INTO lists (order_type, order_format) \
                 VALUES ($1::order_type, $2::order_format) RETURNING id",
            )
            .bind(&pool_dto.order_type)
            .bind(&pool_dto.order_format)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| format!("insert choice list: {e}"))?;

            for (pos, choice) in pool_dto.choices.iter().enumerate() {
                let para_id: i64 =
                    sqlx::query_scalar("INSERT INTO paragraphs DEFAULT VALUES RETURNING id")
                        .fetch_one(&mut *tx)
                        .await
                        .map_err(|e| format!("insert choice paragraph: {e}"))?;

                sqlx::query(
                    "INSERT INTO list_items (list_id, position, paragraph_id) VALUES ($1, $2, $3)",
                )
                .bind(new_list_id)
                .bind(pos as i32)
                .bind(para_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("insert list_item: {e}"))?;

                for (cpos, seg) in choice.content.iter().enumerate() {
                    insert_segment_as_element(&mut *tx, para_id, cpos as i32, seg).await?;
                }
            }

            sqlx::query(
                "UPDATE elemental_questions SET choice_pool_id = $1 WHERE id = $2",
            )
            .bind(new_list_id)
            .bind(&q_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("update choice_pool_id: {e}"))?;
        }
    }

    tx.commit().await.map_err(|e| format!("commit tx: {e}"))?;
    pool.close().await;
    Ok(())
}
