-- ── Remove QuestionSeries (old sub-question grouping mechanism) ───────────────
--
-- All Problems are now ElementalQuestion directly; sub-questions are represented
-- using List content within an ElementalQuestion's content paragraph.
-- This migration clears all existing problem data and drops the obsolete tables.

-- 1. Clear all problem data (must respect FK order)
DELETE FROM problem_group_items;
DELETE FROM single_problems;
DELETE FROM problem_groups;
DELETE FROM elemental_problems;
DELETE FROM question_series_items;
DELETE FROM question_series;
DELETE FROM elemental_questions;

-- 2. Drop QuestionSeries tables
DROP TABLE question_series_items;
DROP TABLE question_series;

-- 3. Remove series_id column and its constraint from elemental_problems
ALTER TABLE elemental_problems
    DROP CONSTRAINT elemental_problem_block_complete,
    DROP COLUMN series_id;

-- 4. Recreate elemental_problem_kind without 'block' and 'plain'
--    PostgreSQL does not support DROP VALUE on enums, so we recreate the type.
ALTER TABLE elemental_problems ALTER COLUMN kind TYPE TEXT;
DROP TYPE elemental_problem_kind;
CREATE TYPE elemental_problem_kind AS ENUM ('question');
ALTER TABLE elemental_problems
    ALTER COLUMN kind TYPE elemental_problem_kind
    USING kind::elemental_problem_kind;

-- 5. Drop the now-redundant index on series_id
DROP INDEX IF EXISTS idx_elemental_problems_series_id;
