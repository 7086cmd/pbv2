import { invoke } from "@tauri-apps/api/core";

export function renderProblemGroup(): Promise<string> {
  return invoke<string>("render_problem_group");
}

export function renderSingleProblem(): Promise<string> {
  return invoke<string>("render_single_problem");
}

export interface CategoryItem {
  id: number;
  curriculum_name: string;
  subject_name: string;
  subject_category: string;
  grade: number;
  categories: string[];
  origin: number | null;
  problem_count: number;
}

export function listCategories(): Promise<CategoryItem[]> {
  return invoke<CategoryItem[]>("list_categories");
}

export interface ProblemListItem {
  id: number;
  preview: string;
}

export function listProblems(categoryId: number): Promise<ProblemListItem[]> {
  return invoke<ProblemListItem[]>("list_problems", { categoryId });
}

export function renderDbProblem(id: number): Promise<string> {
  return invoke<string>("render_db_problem", { id });
}

// ── Editor types ──────────────────────────────────────────────────────────────

export interface TextSegment {
  kind: "text";
  text: string;
  bold: boolean;
  italic: boolean;
  underline: boolean;
  underwave: boolean;
  strikethrough: boolean;
  superscript: boolean;
  subscript: boolean;
  monospace: boolean;
  formula: boolean;
  red: boolean;
}

export interface ImageSegment {
  kind: "image";
  url: string | null;
  data_uri: string | null;
  width_ratio: number | null;
}

export interface ListSegment {
  kind: "list";
  order_type: string;
  order_format: string;
  /** Each item is a flat sequence of Text/Image segments. */
  items: ContentSegment[][];
}

export type ContentSegment = TextSegment | ImageSegment | ListSegment;

export interface ChoiceDto {
  content: ContentSegment[];
}

export interface ChoicePoolDto {
  order_type:
    | "uppercase_alphabetic"
    | "lowercase_alphabetic"
    | "uppercase_roman"
    | "lowercase_roman"
    | "decimal"
    | "unordered";
  order_format: "period" | "parenthesis" | "right_parenthesis" | "none";
  choices: ChoiceDto[];
}

export interface ProblemContentResponse {
  single_problem_id: number;
  question_id: string;
  content: ContentSegment[];
  choice_pool: ChoicePoolDto | null;
}

// ── Editor API ────────────────────────────────────────────────────────────────

export function getProblemContent(id: number): Promise<ProblemContentResponse> {
  return invoke<ProblemContentResponse>("get_problem_content", { id });
}

export function saveProblemContent(
  id: number,
  segments: ContentSegment[],
  choicePool: ChoicePoolDto | null,
): Promise<void> {
  return invoke<void>("save_problem_content", { id, segments, choicePool });
}
