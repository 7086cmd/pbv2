-- ── Enum types ────────────────────────────────────────────────────────────────

CREATE TYPE font_size AS ENUM (
    'tiny',
    'script',
    'footnote',
    'small',
    'normal',
    'large',
    'x_large',
    'xx_large'
);

CREATE TYPE image_format AS ENUM ('png', 'jpeg');

-- Discriminator for the Image variant.
CREATE TYPE image_kind AS ENUM ('binary', 'url', 'latex', 'pdf_svg');

CREATE TYPE order_type AS ENUM (
    'uppercase_alphabetic',
    'lowercase_alphabetic',
    'uppercase_roman',
    'lowercase_roman',
    'decimal',
    'unordered'
);

CREATE TYPE order_format AS ENUM (
    'period',
    'parenthesis',
    'right_parenthesis',
    'none'
);

-- Discriminator for the Element variant.
CREATE TYPE element_kind AS ENUM ('text', 'table', 'image', 'list', 'blank');

-- ── Text ──────────────────────────────────────────────────────────────────────
--
-- `formatting` is a packed sequence of 8-byte TextFormat segments:
--   bytes 0-1  : language (u16 LE)
--   bytes 2-5  : start    (u32 LE) — byte offset into `content`
--   bytes 6-7  : flags    (u16 LE) — TextFlags bitflags
--
-- `content` is the raw UTF-8 payload of all text runs concatenated.
-- Run boundaries are derived from the `start` pointers in `formatting`.
--
-- `color_r/g/b` are stored as SMALLINT (0-255) to avoid a bespoke domain.

CREATE TABLE texts (
    id          BIGSERIAL   PRIMARY KEY,
    formatting  BYTEA       NOT NULL,
    content     BYTEA       NOT NULL,
    font_size   font_size   NOT NULL DEFAULT 'normal',
    color_r     SMALLINT    NOT NULL DEFAULT 0 CHECK (color_r BETWEEN 0 AND 255),
    color_g     SMALLINT    NOT NULL DEFAULT 0 CHECK (color_g BETWEEN 0 AND 255),
    color_b     SMALLINT    NOT NULL DEFAULT 0 CHECK (color_b BETWEEN 0 AND 255)
);

-- ── Image ─────────────────────────────────────────────────────────────────────
--
-- Single-table discriminator; exactly one variant block is populated per row.
--
--   Binary  → buffer, format, filename, width_ratio
--   Url     → url
--   Latex   → tex_code, svg_content, png_content  (CompiledGraph inline)
--   PdfSvg  → pdf_buffer, svg_content, width_ratio
--
-- `svg_content` is shared by the Latex and PdfSvg variants.
-- `width_ratio` is shared by the Binary and PdfSvg variants.

CREATE TABLE images (
    id          BIGSERIAL           PRIMARY KEY,
    kind        image_kind          NOT NULL,

    -- Binary variant
    buffer      BYTEA,
    format      image_format,
    filename    TEXT,

    -- Url variant
    url         TEXT,

    -- Latex (CompiledGraph) variant
    tex_code    TEXT,
    png_content BYTEA,

    -- Shared by Latex and PdfSvg
    svg_content TEXT,

    -- Shared by Binary and PdfSvg
    width_ratio DOUBLE PRECISION,

    -- PdfSvg variant
    pdf_buffer  BYTEA,

    CONSTRAINT images_binary_complete
        CHECK (kind <> 'binary'  OR (buffer IS NOT NULL AND format IS NOT NULL)),
    CONSTRAINT images_url_complete
        CHECK (kind <> 'url'     OR url IS NOT NULL),
    CONSTRAINT images_latex_complete
        CHECK (kind <> 'latex'   OR tex_code IS NOT NULL),
    CONSTRAINT images_pdf_svg_complete
        CHECK (kind <> 'pdf_svg' OR (pdf_buffer IS NOT NULL AND svg_content IS NOT NULL))
);

-- ── Blank ─────────────────────────────────────────────────────────────────────
--
-- `id` is application-assigned (matches Blank::id: i32) — no sequence.
-- `answer_id` references the Text that holds the correct answer.

CREATE TABLE blanks (
    id          INTEGER     PRIMARY KEY,
    mark        REAL        NOT NULL CHECK (mark >= 0),
    answer_id   BIGINT      NOT NULL REFERENCES texts (id),
    width       REAL        NOT NULL CHECK (width >= 0)
);

-- ── Table + Cells ─────────────────────────────────────────────────────────────
--
-- `table_cells` uses a composite PK (table_id, row, col) matching the Rust
-- HashMap key `(usize, usize)`.  Covered cells (those spanned over by a
-- neighbour) are simply absent — the application derives them at render time.

CREATE TABLE tables (
    id      BIGSERIAL   PRIMARY KEY,
    rows    INTEGER     NOT NULL CHECK (rows > 0),
    cols    INTEGER     NOT NULL CHECK (cols > 0)
);

CREATE TABLE table_cells (
    table_id    BIGINT      NOT NULL REFERENCES tables (id) ON DELETE CASCADE,
    row         INTEGER     NOT NULL CHECK (row >= 0),
    col         INTEGER     NOT NULL CHECK (col >= 0),
    content_id  BIGINT      NOT NULL REFERENCES texts (id),
    row_span    INTEGER     NOT NULL DEFAULT 1 CHECK (row_span >= 1),
    col_span    INTEGER     NOT NULL DEFAULT 1 CHECK (col_span >= 1),
    header      BOOLEAN     NOT NULL DEFAULT FALSE,

    PRIMARY KEY (table_id, row, col)
);

-- ── List ──────────────────────────────────────────────────────────────────────

CREATE TABLE lists (
    id           BIGSERIAL    PRIMARY KEY,
    order_type   order_type   NOT NULL,
    order_format order_format NOT NULL
);

-- ── Paragraph ─────────────────────────────────────────────────────────────────

CREATE TABLE paragraphs (
    id  BIGSERIAL   PRIMARY KEY
);

-- `list_items` is the ordered sequence of Paragraphs belonging to a List.
-- `position` is 0-based and forms a gapless sequence per list.

CREATE TABLE list_items (
    list_id      BIGINT  NOT NULL REFERENCES lists      (id) ON DELETE CASCADE,
    position     INTEGER NOT NULL CHECK (position >= 0),
    paragraph_id BIGINT  NOT NULL REFERENCES paragraphs (id),

    PRIMARY KEY (list_id, position)
);

-- ── Element ───────────────────────────────────────────────────────────────────
--
-- Discriminated union.  Exactly one FK column is non-NULL, matching `kind`.

CREATE TABLE elements (
    id          BIGSERIAL       PRIMARY KEY,
    kind        element_kind    NOT NULL,

    text_id     BIGINT          REFERENCES texts      (id),
    table_id    BIGINT          REFERENCES tables     (id),
    image_id    BIGINT          REFERENCES images     (id),
    list_id     BIGINT          REFERENCES lists      (id),
    blank_id    INTEGER         REFERENCES blanks     (id),

    CONSTRAINT element_text_complete
        CHECK (kind <> 'text'  OR text_id  IS NOT NULL),
    CONSTRAINT element_table_complete
        CHECK (kind <> 'table' OR table_id IS NOT NULL),
    CONSTRAINT element_image_complete
        CHECK (kind <> 'image' OR image_id IS NOT NULL),
    CONSTRAINT element_list_complete
        CHECK (kind <> 'list'  OR list_id  IS NOT NULL),
    CONSTRAINT element_blank_complete
        CHECK (kind <> 'blank' OR blank_id IS NOT NULL)
);

-- `paragraph_elements` is the ordered sequence of Elements inside a Paragraph.
-- `position` is 0-based and forms a gapless sequence per paragraph.

CREATE TABLE paragraph_elements (
    paragraph_id BIGINT  NOT NULL REFERENCES paragraphs (id) ON DELETE CASCADE,
    position     INTEGER NOT NULL CHECK (position >= 0),
    element_id   BIGINT  NOT NULL REFERENCES elements   (id),

    PRIMARY KEY (paragraph_id, position)
);

-- ── Indexes ───────────────────────────────────────────────────────────────────

CREATE INDEX idx_table_cells_table_id  ON table_cells         (table_id);
CREATE INDEX idx_list_items_list_id    ON list_items          (list_id);
CREATE INDEX idx_para_elems_para_id    ON paragraph_elements  (paragraph_id);
CREATE INDEX idx_elements_text_id      ON elements            (text_id)  WHERE text_id  IS NOT NULL;
CREATE INDEX idx_elements_table_id     ON elements            (table_id) WHERE table_id IS NOT NULL;
CREATE INDEX idx_elements_image_id     ON elements            (image_id) WHERE image_id IS NOT NULL;
CREATE INDEX idx_elements_list_id      ON elements            (list_id)  WHERE list_id  IS NOT NULL;
CREATE INDEX idx_elements_blank_id     ON elements            (blank_id) WHERE blank_id IS NOT NULL;
