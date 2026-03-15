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
