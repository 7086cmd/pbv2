#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Cirriculum {
    pub id: i32,
    pub name: String,
    /// Instructional language, not the language of the subject matter. For example, a subject about English literature could be taught in English or in another language.
    /// The ISO 639-1 code for the language, e.g. "en-US" for English (United States), "zh-CN" for Chinese (mainland China), etc.
    pub instruction_language: String,
    /// If true, the cirriculum is designed for international use and may include subjects that are not specific to any particular country or region. If false, the cirriculum is designed for a specific country or region and may include subjects that are specific to that country or region.
    /// For example, AP, IB, and A-level cirriculums are international, while the Gaokao (Chinese national college entrance exam) cirriculum, Hong Kong DSE cirriculum, and US high school cirriculum are not international.
    pub international: bool,
}

#[derive(Debug, Clone)]
pub struct Subject {
    pub id: i32,
    pub name: String,
    pub category: SubjectCategory,
    pub cirriculum_id: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SubjectCategory {
    Language,
    STEM,
    Humanities,
    Arts,
    Other,
}
