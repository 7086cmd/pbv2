-- ── Enum types ────────────────────────────────────────────────────────────────

--- Discriminator for the QuestionBlock variant (answer-space type on a question).
CREATE TYPE question_block_kind AS ENUM ('essay', 'proof', 'solve', 'none');

-- Discriminator for the ElementalProblem variant.
CREATE TYPE elemental_problem_kind AS ENUM ('question', 'block', 'plain');

-- ── problem_categories ────────────────────────────────────────────────────────
--
-- Metadata about a problem's curriculum placement and origin.
-- `categories` is a free-form tag array (e.g. ["Algebra", "Quadratics"]).
-- `origin` is an optional FK into an external problem_origin table.

CREATE TABLE problem_categories (
    id          BIGSERIAL   PRIMARY KEY,
    cirriculum  INTEGER     NOT NULL,
    subject     INTEGER     NOT NULL,
    grade       INTEGER     NOT NULL,
    categories  TEXT[]      NOT NULL DEFAULT '{}',
    origin      INTEGER
);

-- ── elemental_questions ───────────────────────────────────────────────────────
--
-- One row per ElementalQuestion.  The `id` column is the application-assigned
-- string identifier (e.g. "q1a").
--
-- Block columns:
--   essay  → block_lines (number of ruled answer lines)
--   proof  → block_space (vertical space in rem)
--   solve  → block_space (vertical space in rem)
--   none   → both NULL

CREATE TABLE elemental_questions (
    id              TEXT                    PRIMARY KEY,
    content_id      BIGINT                  NOT NULL REFERENCES paragraphs (id),
    answer_id       BIGINT                  REFERENCES paragraphs (id),
    solution_id     BIGINT                  REFERENCES paragraphs (id),
    choice_pool_id  BIGINT                  REFERENCES lists      (id),
    block_kind      question_block_kind     NOT NULL DEFAULT 'none',
    block_lines     INTEGER                 CHECK (block_lines > 0),
    block_space     REAL                    CHECK (block_space >= 0),

    CONSTRAINT question_essay_complete
        CHECK (block_kind <> 'essay' OR block_lines IS NOT NULL),
    CONSTRAINT question_proof_complete
        CHECK (block_kind <> 'proof' OR block_space IS NOT NULL),
    CONSTRAINT question_solve_complete
        CHECK (block_kind <> 'solve' OR block_space IS NOT NULL)
);

-- ── question_series ───────────────────────────────────────────────────────────

CREATE TABLE question_series (
    id              BIGSERIAL       PRIMARY KEY,
    content_id      BIGINT          NOT NULL REFERENCES paragraphs   (id),
    order_type      order_type      NOT NULL,
    order_format    order_format    NOT NULL,
    order_resume    BOOLEAN         NOT NULL DEFAULT FALSE
);

-- Ordered sub-questions belonging to a QuestionSeries.  `position` is 0-based.

CREATE TABLE question_series_items (
    series_id   BIGINT  NOT NULL REFERENCES question_series      (id) ON DELETE CASCADE,
    position    INTEGER NOT NULL CHECK (position >= 0),
    question_id TEXT    NOT NULL REFERENCES elemental_questions  (id),

    PRIMARY KEY (series_id, position)
);

-- ── elemental_problems ────────────────────────────────────────────────────────
--
-- Discriminated union: exactly one FK column is non-NULL, matching `kind`.

CREATE TABLE elemental_problems (
    id          BIGSERIAL               PRIMARY KEY,
    kind        elemental_problem_kind  NOT NULL,
    question_id TEXT                    REFERENCES elemental_questions (id),
    series_id   BIGINT                  REFERENCES question_series     (id),

    CONSTRAINT elemental_problem_question_complete
        CHECK (kind <> 'question' OR question_id IS NOT NULL),
    CONSTRAINT elemental_problem_block_complete
        CHECK (kind <> 'block'    OR series_id   IS NOT NULL)
);

-- ── single_problems ───────────────────────────────────────────────────────────

CREATE TABLE single_problems (
    id          BIGSERIAL   PRIMARY KEY,
    problem_id  BIGINT      NOT NULL REFERENCES elemental_problems  (id),
    category_id BIGINT      NOT NULL REFERENCES problem_categories  (id)
);

-- ── problem_groups ────────────────────────────────────────────────────────────

CREATE TABLE problem_groups (
    id          BIGSERIAL   PRIMARY KEY,
    material_id BIGINT      NOT NULL REFERENCES paragraphs          (id),
    category_id BIGINT      NOT NULL REFERENCES problem_categories  (id)
);

-- Ordered ElementalProblems belonging to a ProblemGroup.  `position` is 0-based.

CREATE TABLE problem_group_items (
    group_id    BIGINT  NOT NULL REFERENCES problem_groups      (id) ON DELETE CASCADE,
    position    INTEGER NOT NULL CHECK (position >= 0),
    problem_id  BIGINT  NOT NULL REFERENCES elemental_problems  (id),

    PRIMARY KEY (group_id, position)
);

-- ── Indexes ───────────────────────────────────────────────────────────────────

CREATE INDEX idx_q_series_items_series_id ON question_series_items (series_id);
CREATE INDEX idx_prob_group_items_group_id ON problem_group_items   (group_id);
CREATE INDEX idx_elemental_problems_question_id
    ON elemental_problems (question_id) WHERE question_id IS NOT NULL;
CREATE INDEX idx_elemental_problems_series_id
    ON elemental_problems (series_id)   WHERE series_id   IS NOT NULL;
