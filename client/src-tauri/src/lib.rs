mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::render_problem_group,
            commands::render_single_problem,
            commands::list_categories,
            commands::list_problems,
            commands::render_db_problem,
            commands::get_problem_content,
            commands::save_problem_content,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
