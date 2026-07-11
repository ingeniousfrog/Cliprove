mod app_state;
mod commands;
mod db;
mod errors;
mod logging;
mod mock;
mod models;
mod sidecar;
mod tasks;

use std::path::PathBuf;

use app_state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    logging::init_logging();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data_dir = app_data_dir(app.handle())?;
            let database = db::Database::open(&app_data_dir)?;
            database.tasks().recover_interrupted()?;
            let settings = database.settings().get_all()?;
            std::fs::create_dir_all(&settings.download_directory).ok();
            app.manage(AppState::new(database));
            tracing::info!("Cliprove started, db at {:?}", app_data_dir);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::parse_link,
            commands::search_media,
            commands::create_download_spec,
            commands::enqueue_download,
            commands::list_tasks,
            commands::task_action,
            commands::list_library,
            commands::get_settings,
            commands::update_settings,
            commands::validate_platform_auth,
            commands::start_sidecar,
            commands::sidecar_health,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn app_data_dir(app: &tauri::AppHandle) -> tauri::Result<PathBuf> {
    let dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}
