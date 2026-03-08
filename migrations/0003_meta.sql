-- ── Subject category enum ─────────────────────────────────────────────────────

CREATE TYPE subject_category AS ENUM (
    'language',
    'stem',
    'humanities',
    'arts',
    'other'
);

-- ── Curriculums ───────────────────────────────────────────────────────────────
--
-- Referenced by problem_categories.cirriculum (INTEGER FK).
-- instruction_language is an ISO 639-1 tag, e.g. "en-US", "zh-CN".

CREATE TABLE curriculums (
    id                   SERIAL   PRIMARY KEY,
    name                 TEXT     NOT NULL,
    instruction_language TEXT     NOT NULL,
    international        BOOLEAN  NOT NULL DEFAULT FALSE
);

-- ── Subjects ──────────────────────────────────────────────────────────────────
--
-- Referenced by problem_categories.subject (INTEGER FK).

CREATE TABLE subjects (
    id            SERIAL           PRIMARY KEY,
    name          TEXT             NOT NULL,
    category      subject_category NOT NULL,
    cirriculum_id INTEGER          NOT NULL REFERENCES curriculums (id)
);

-- ── Problem origins ───────────────────────────────────────────────────────────
--
-- Tracks the source exam of a problem, e.g. "2024 Gaokao Paper 1".
-- Referenced by problem_categories.origin (INTEGER FK, optional).

CREATE TABLE problem_origins (
    id      SERIAL  PRIMARY KEY,
    name    TEXT    NOT NULL,
    year    INTEGER,
    notes   TEXT
);

-- ── Back-fill foreign keys on problem_categories ──────────────────────────────
--
-- These columns existed as plain integers; now that the referenced tables
-- exist we can add proper FK constraints.

ALTER TABLE problem_categories
    ADD CONSTRAINT fk_pc_cirriculum
        FOREIGN KEY (cirriculum) REFERENCES curriculums    (id),
    ADD CONSTRAINT fk_pc_subject
        FOREIGN KEY (subject)    REFERENCES subjects       (id),
    ADD CONSTRAINT fk_pc_origin
        FOREIGN KEY (origin)     REFERENCES problem_origins (id);

-- ── Plain variant FK on elemental_problems ────────────────────────────────────
--
-- The Rust schema has ElementalProblem::Plain(Paragraph) but the original
-- migration left no column to reference the paragraph.  Add it now.

ALTER TABLE elemental_problems
    ADD COLUMN paragraph_id BIGINT REFERENCES paragraphs (id),
    ADD CONSTRAINT elemental_problem_plain_complete
        CHECK (kind <> 'plain' OR paragraph_id IS NOT NULL);

-- ── Indexes ───────────────────────────────────────────────────────────────────

CREATE INDEX idx_subjects_cirriculum_id ON subjects (cirriculum_id);
CREATE INDEX idx_elemental_problems_paragraph_id
    ON elemental_problems (paragraph_id) WHERE paragraph_id IS NOT NULL;

-- ── Seed data: Gaokao ─────────────────────────────────────────────────────────
--
-- 普通高等学校招生全国统一考试 (Gaokao) — the Chinese national college-entrance
-- examination.  Instruction language: Simplified Chinese (zh-CN).
-- Not international.

INSERT INTO curriculums (id, name, instruction_language, international)
VALUES (1, '普通高等学校招生全国统一考试', 'zh-CN', FALSE);

-- STEM & Language subjects.  Liberal arts (History, Geography, Politics) are
-- intentionally omitted for now.

INSERT INTO subjects (id, name, category, cirriculum_id) VALUES
    (1,  '语文',     'language', 1),   -- Chinese Language & Literature
    (2,  '数学',     'stem',     1),   -- Mathematics
    (3,  '英语',     'language', 1),   -- English
    (4,  '物理',     'stem',     1),   -- Physics
    (5,  '化学',     'stem',     1),   -- Chemistry
    (6,  '生物学',   'stem',     1),   -- Biology
    (7,  '信息技术', 'stem',     1),   -- Information Technology
    (8,  '通用技术', 'stem',     1);   -- General Technology

-- Keep sequences in sync after explicit-id inserts.
SELECT setval('curriculums_id_seq', (SELECT MAX(id) FROM curriculums));
SELECT setval('subjects_id_seq',    (SELECT MAX(id) FROM subjects));
