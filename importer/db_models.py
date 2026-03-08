"""
SQLAlchemy ORM models mirroring the PostgreSQL schema defined in migrations/.

Tables covered
──────────────
  migrations/0001_initial.sql : texts, images, blanks, tables, table_cells,
                                 lists, list_items, paragraphs, elements,
                                 paragraph_elements
  migrations/0002_problems.sql: problem_categories, elemental_questions,
                                 question_series, question_series_items,
                                 elemental_problems, single_problems,
                                 problem_groups, problem_group_items
  migrations/0003_meta.sql    : curriculums, subjects, problem_origins
                                 (+ ALTER TABLE patches on the above)
"""

from __future__ import annotations

import enum

from sqlalchemy import (
    ARRAY,
    BigInteger,
    Boolean,
    Column,
    Enum,
    Float,
    ForeignKey,
    Integer,
    LargeBinary,
    REAL,
    SmallInteger,
    Text,
)
from sqlalchemy.orm import DeclarativeBase


class Base(DeclarativeBase):
    pass


# ══════════════════════════════════════════════════════════════════════════════
# Python enums (mirror PostgreSQL CREATE TYPE … AS ENUM)
# ══════════════════════════════════════════════════════════════════════════════


class FontSizeEnum(str, enum.Enum):
    tiny = "tiny"
    script = "script"
    footnote = "footnote"
    small = "small"
    normal = "normal"
    large = "large"
    x_large = "x_large"
    xx_large = "xx_large"


class ImageFormatEnum(str, enum.Enum):
    png = "png"
    jpeg = "jpeg"


class ImageKindEnum(str, enum.Enum):
    binary = "binary"
    url = "url"
    latex = "latex"
    pdf_svg = "pdf_svg"


class OrderTypeEnum(str, enum.Enum):
    uppercase_alphabetic = "uppercase_alphabetic"
    lowercase_alphabetic = "lowercase_alphabetic"
    uppercase_roman = "uppercase_roman"
    lowercase_roman = "lowercase_roman"
    decimal = "decimal"
    unordered = "unordered"


class OrderFormatEnum(str, enum.Enum):
    period = "period"
    parenthesis = "parenthesis"
    right_parenthesis = "right_parenthesis"
    none = "none"


class ElementKindEnum(str, enum.Enum):
    text = "text"
    table = "table"
    image = "image"
    list = "list"
    blank = "blank"


class QuestionBlockKindEnum(str, enum.Enum):
    essay = "essay"
    proof = "proof"
    solve = "solve"
    none = "none"


class ElementalProblemKindEnum(str, enum.Enum):
    question = "question"
    block = "block"
    plain = "plain"


class SubjectCategoryEnum(str, enum.Enum):
    language = "language"
    stem = "stem"
    humanities = "humanities"
    arts = "arts"
    other = "other"


# ══════════════════════════════════════════════════════════════════════════════
# 0003_meta: Curriculum / Subject / Origin
# ══════════════════════════════════════════════════════════════════════════════


class Curriculum(Base):
    __tablename__ = "curriculums"

    id = Column(Integer, primary_key=True, autoincrement=True)
    name = Column(Text, nullable=False)
    instruction_language = Column(Text, nullable=False)
    international = Column(Boolean, nullable=False, default=False)


class Subject(Base):
    __tablename__ = "subjects"

    id = Column(Integer, primary_key=True, autoincrement=True)
    name = Column(Text, nullable=False)
    category = Column(
        Enum(SubjectCategoryEnum, name="subject_category"),
        nullable=False,
    )
    cirriculum_id = Column(Integer, ForeignKey("curriculums.id"), nullable=False)


class ProblemOrigin(Base):
    __tablename__ = "problem_origins"

    id = Column(Integer, primary_key=True, autoincrement=True)
    name = Column(Text, nullable=False)
    year = Column(Integer)
    notes = Column(Text)


# ══════════════════════════════════════════════════════════════════════════════
# 0001_initial: Core element tables
# ══════════════════════════════════════════════════════════════════════════════


class TextRow(Base):
    """Row in the ``texts`` table."""

    __tablename__ = "texts"

    id = Column(BigInteger, primary_key=True, autoincrement=True)
    formatting = Column(LargeBinary, nullable=False)
    content = Column(LargeBinary, nullable=False)
    font_size = Column(
        Enum(FontSizeEnum, name="font_size"),
        nullable=False,
        default=FontSizeEnum.normal,
    )
    color_r = Column(SmallInteger, nullable=False, default=0)
    color_g = Column(SmallInteger, nullable=False, default=0)
    color_b = Column(SmallInteger, nullable=False, default=0)


class ImageRow(Base):
    """Row in the ``images`` table (discriminated union)."""

    __tablename__ = "images"

    id = Column(BigInteger, primary_key=True, autoincrement=True)
    kind = Column(Enum(ImageKindEnum, name="image_kind"), nullable=False)

    # Binary variant
    buffer = Column(LargeBinary)
    format = Column(Enum(ImageFormatEnum, name="image_format"))
    filename = Column(Text)

    # URL variant
    url = Column(Text)

    # Latex (CompiledGraph) variant
    tex_code = Column(Text)
    png_content = Column(LargeBinary)

    # Shared by Latex and PdfSvg
    svg_content = Column(Text)

    # Shared by Binary and PdfSvg
    width_ratio = Column(Float)

    # PdfSvg variant
    pdf_buffer = Column(LargeBinary)


class BlankRow(Base):
    """Row in the ``blanks`` table."""

    __tablename__ = "blanks"

    id = Column(Integer, primary_key=True)
    mark = Column(REAL, nullable=False)
    answer_id = Column(BigInteger, ForeignKey("texts.id"), nullable=False)
    width = Column(REAL, nullable=False)


class TableRow(Base):
    """Row in the ``tables`` table."""

    __tablename__ = "tables"

    id = Column(BigInteger, primary_key=True, autoincrement=True)
    rows = Column(Integer, nullable=False)
    cols = Column(Integer, nullable=False)


class TableCell(Base):
    """Row in the ``table_cells`` table."""

    __tablename__ = "table_cells"

    table_id = Column(
        BigInteger,
        ForeignKey("tables.id", ondelete="CASCADE"),
        primary_key=True,
    )
    row = Column(Integer, primary_key=True)
    col = Column(Integer, primary_key=True)
    content_id = Column(BigInteger, ForeignKey("texts.id"), nullable=False)
    row_span = Column(Integer, nullable=False, default=1)
    col_span = Column(Integer, nullable=False, default=1)
    header = Column(Boolean, nullable=False, default=False)


class ListRow(Base):
    """Row in the ``lists`` table."""

    __tablename__ = "lists"

    id = Column(BigInteger, primary_key=True, autoincrement=True)
    order_type = Column(Enum(OrderTypeEnum, name="order_type"), nullable=False)
    order_format = Column(Enum(OrderFormatEnum, name="order_format"), nullable=False)


class Paragraph(Base):
    """Row in the ``paragraphs`` table."""

    __tablename__ = "paragraphs"

    id = Column(BigInteger, primary_key=True, autoincrement=True)


class ListItem(Base):
    """Row in the ``list_items`` table."""

    __tablename__ = "list_items"

    list_id = Column(
        BigInteger,
        ForeignKey("lists.id", ondelete="CASCADE"),
        primary_key=True,
    )
    position = Column(Integer, primary_key=True)
    paragraph_id = Column(BigInteger, ForeignKey("paragraphs.id"), nullable=False)


class Element(Base):
    """Row in the ``elements`` table (discriminated union)."""

    __tablename__ = "elements"

    id = Column(BigInteger, primary_key=True, autoincrement=True)
    kind = Column(Enum(ElementKindEnum, name="element_kind"), nullable=False)
    text_id = Column(BigInteger, ForeignKey("texts.id"))
    table_id = Column(BigInteger, ForeignKey("tables.id"))
    image_id = Column(BigInteger, ForeignKey("images.id"))
    list_id = Column(BigInteger, ForeignKey("lists.id"))
    blank_id = Column(Integer, ForeignKey("blanks.id"))


class ParagraphElement(Base):
    """Row in the ``paragraph_elements`` table."""

    __tablename__ = "paragraph_elements"

    paragraph_id = Column(
        BigInteger,
        ForeignKey("paragraphs.id", ondelete="CASCADE"),
        primary_key=True,
    )
    position = Column(Integer, primary_key=True)
    element_id = Column(BigInteger, ForeignKey("elements.id"), nullable=False)


# ══════════════════════════════════════════════════════════════════════════════
# 0002_problems: Problem tables
# ══════════════════════════════════════════════════════════════════════════════


class ProblemCategory(Base):
    """Row in the ``problem_categories`` table."""

    __tablename__ = "problem_categories"

    id = Column(BigInteger, primary_key=True, autoincrement=True)
    cirriculum = Column(Integer, ForeignKey("curriculums.id"), nullable=False)
    subject = Column(Integer, ForeignKey("subjects.id"), nullable=False)
    grade = Column(Integer, nullable=False)
    categories = Column(ARRAY(Text), nullable=False, default=list)
    origin = Column(Integer, ForeignKey("problem_origins.id"))


class ElementalQuestion(Base):
    """Row in the ``elemental_questions`` table."""

    __tablename__ = "elemental_questions"

    id = Column(Text, primary_key=True)
    content_id = Column(BigInteger, ForeignKey("paragraphs.id"), nullable=False)
    answer_id = Column(BigInteger, ForeignKey("paragraphs.id"))
    solution_id = Column(BigInteger, ForeignKey("paragraphs.id"))
    choice_pool_id = Column(BigInteger, ForeignKey("lists.id"))
    block_kind = Column(
        Enum(QuestionBlockKindEnum, name="question_block_kind"),
        nullable=False,
        default=QuestionBlockKindEnum.none,
    )
    block_lines = Column(Integer)
    block_space = Column(REAL)


class QuestionSeries(Base):
    """Row in the ``question_series`` table."""

    __tablename__ = "question_series"

    id = Column(BigInteger, primary_key=True, autoincrement=True)
    content_id = Column(BigInteger, ForeignKey("paragraphs.id"), nullable=False)
    order_type = Column(Enum(OrderTypeEnum, name="order_type"), nullable=False)
    order_format = Column(Enum(OrderFormatEnum, name="order_format"), nullable=False)
    order_resume = Column(Boolean, nullable=False, default=False)


class QuestionSeriesItem(Base):
    """Row in the ``question_series_items`` table."""

    __tablename__ = "question_series_items"

    series_id = Column(
        BigInteger,
        ForeignKey("question_series.id", ondelete="CASCADE"),
        primary_key=True,
    )
    position = Column(Integer, primary_key=True)
    question_id = Column(Text, ForeignKey("elemental_questions.id"), nullable=False)


class ElementalProblem(Base):
    """
    Row in the ``elemental_problems`` table (discriminated union).

    paragraph_id column added by migration 0003 for the ``plain`` variant.
    """

    __tablename__ = "elemental_problems"

    id = Column(BigInteger, primary_key=True, autoincrement=True)
    kind = Column(
        Enum(ElementalProblemKindEnum, name="elemental_problem_kind"),
        nullable=False,
    )
    question_id = Column(Text, ForeignKey("elemental_questions.id"))
    series_id = Column(BigInteger, ForeignKey("question_series.id"))
    paragraph_id = Column(BigInteger, ForeignKey("paragraphs.id"))  # plain variant


class SingleProblem(Base):
    """Row in the ``single_problems`` table."""

    __tablename__ = "single_problems"

    id = Column(BigInteger, primary_key=True, autoincrement=True)
    problem_id = Column(BigInteger, ForeignKey("elemental_problems.id"), nullable=False)
    category_id = Column(BigInteger, ForeignKey("problem_categories.id"), nullable=False)


class ProblemGroup(Base):
    """Row in the ``problem_groups`` table."""

    __tablename__ = "problem_groups"

    id = Column(BigInteger, primary_key=True, autoincrement=True)
    material_id = Column(BigInteger, ForeignKey("paragraphs.id"), nullable=False)
    category_id = Column(BigInteger, ForeignKey("problem_categories.id"), nullable=False)


class ProblemGroupItem(Base):
    """Row in the ``problem_group_items`` table."""

    __tablename__ = "problem_group_items"

    group_id = Column(
        BigInteger,
        ForeignKey("problem_groups.id", ondelete="CASCADE"),
        primary_key=True,
    )
    position = Column(Integer, primary_key=True)
    problem_id = Column(BigInteger, ForeignKey("elemental_problems.id"), nullable=False)
