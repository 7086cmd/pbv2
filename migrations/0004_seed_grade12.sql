-- ── Grade-12 problem categories — Gaokao, mock & real ────────────────────────
--
-- Two categories per subject:
--   categories = '{mock}'  – practice / mock exam problems
--   categories = '{real}'  – actual Gaokao (college-entrance) exam problems
--
-- Cirriculum id=1 (普通高等学校招生全国统一考试), grade=12.
-- Subject ids 1-8 match the subjects seeded in migration 0003.
--
-- Resulting IDs:
--   odd  ids  1, 3, 5, 7,  9, 11, 13, 15  → mock
--   even ids  2, 4, 6, 8, 10, 12, 14, 16  → real

INSERT INTO problem_categories (id, cirriculum, subject, grade, categories) VALUES
    -- 语文 (Chinese)
    ( 1, 1, 1, 12, '{mock}'),
    ( 2, 1, 1, 12, '{real}'),
    -- 数学 (Mathematics)
    ( 3, 1, 2, 12, '{mock}'),
    ( 4, 1, 2, 12, '{real}'),
    -- 英语 (English)
    ( 5, 1, 3, 12, '{mock}'),
    ( 6, 1, 3, 12, '{real}'),
    -- 物理 (Physics)
    ( 7, 1, 4, 12, '{mock}'),
    ( 8, 1, 4, 12, '{real}'),
    -- 化学 (Chemistry)
    ( 9, 1, 5, 12, '{mock}'),
    (10, 1, 5, 12, '{real}'),
    -- 生物学 (Biology)
    (11, 1, 6, 12, '{mock}'),
    (12, 1, 6, 12, '{real}'),
    -- 信息技术 (Information Technology)
    (13, 1, 7, 12, '{mock}'),
    (14, 1, 7, 12, '{real}'),
    -- 通用技术 (General Technology)
    (15, 1, 8, 12, '{mock}'),
    (16, 1, 8, 12, '{real}');

-- Keep sequence in sync after explicit-id inserts.
SELECT setval('problem_categories_id_seq', (SELECT MAX(id) FROM problem_categories));
