#[derive(Debug, Clone)]
pub struct ProblemCategory {
    pub cirriculum: i32, // refer to cirriculum table
    pub subject: i32, // refer to subject table whose cirriculum_id matches the cirriculum field
    pub grade: i32, // just the grade number, e.g. 10 for 10th grade
    pub categories: Vec<String>,
    pub origin: Option<i32> // refer to problem_origin table, optional
}
