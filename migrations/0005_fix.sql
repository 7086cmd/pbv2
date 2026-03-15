-- ── Fix for 0005: clean up remaining question_series artifacts ────────────────

-- 1. Drop question_series (series_id FK already removed, CASCADE for safety)
DROP TABLE question_series CASCADE;

-- 2. Drop CHECK constraints that reference the enum (required before type change)
ALTER TABLE elemental_problems
    DROP CONSTRAINT elemental_problem_question_complete,
    DROP CONSTRAINT elemental_problem_plain_complete,
    DROP COLUMN paragraph_id;

-- 3. Recreate elemental_problem_kind with only 'question'
ALTER TABLE elemental_problems ALTER COLUMN kind TYPE TEXT;
DROP TYPE elemental_problem_kind;
CREATE TYPE elemental_problem_kind AS ENUM ('question');
ALTER TABLE elemental_problems
    ALTER COLUMN kind TYPE elemental_problem_kind
    USING kind::elemental_problem_kind;

-- 4. Re-add the question constraint
ALTER TABLE elemental_problems
    ADD CONSTRAINT elemental_problem_question_complete
        CHECK (kind <> 'question' OR question_id IS NOT NULL);

-- 5. Drop the index that was for paragraph_id (plain variant)
DROP INDEX IF EXISTS idx_elemental_problems_paragraph_id;
