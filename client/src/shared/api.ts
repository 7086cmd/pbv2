import { invoke } from '@tauri-apps/api/core'

export function renderProblemGroup(): Promise<string> {
  return invoke<string>('render_problem_group')
}

export function renderSingleProblem(): Promise<string> {
  return invoke<string>('render_single_problem')
}
