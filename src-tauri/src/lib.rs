mod app_state;
pub mod db;
mod commands;
mod errors;
mod logging;
mod mock;
pub mod models;
mod shell;
mod sidecar;
mod tasks;

use std::path::PathBuf;
use std::sync::Arc;

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
            if let Some(resolved) = shell::resolve_ffmpeg_path(&settings.ffmpeg_path) {
                let resolved_str = resolved.to_string_lossy().to_string();
                if resolved_str != settings.ffmpeg_path {
                    let mut partial = settings;
                    partial.ffmpeg_path = resolved_str;
                    let _ = database.settings().update(&partial);
                }
            }
            let state = Arc::new(AppState::new(database));
            let sidecar = Arc::clone(&state.sidecar);
            app.manage(state);
            tracing::info!("Cliprove started, db at {:?}", app_data_dir);

            // Sidecar cold start (PyInstaller onefile unpack) can take 30–60s.
            // Spawn in background so the window appears immediately.
            std::thread::spawn(move || {
                if let Err(error) = sidecar.start() {
                    tracing::warn!("Sidecar auto-start failed: {error}");
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::parse_link,
            commands::search_media,
            commands::create_download_spec,
            commands::enqueue_download,
            commands::enqueue_download_batch,
            commands::list_tasks,
            commands::task_action,
            commands::list_library,
            commands::get_library_item,
            commands::delete_library_item,
            commands::list_tags,
            commands::create_tag,
            commands::delete_tag,
            commands::set_library_tags,
            commands::list_collections,
            commands::create_collection,
            commands::rename_collection,
            commands::delete_collection,
            commands::add_to_collection,
            commands::remove_from_collection,
            commands::reveal_in_finder,
            commands::open_local_file,
            commands::read_local_file,
            commands::validate_ffmpeg,
            commands::ensure_ffmpeg,
            commands::count_library,
            commands::get_app_paths,
            commands::get_settings,
            commands::update_settings,
            commands::validate_platform_auth,
            commands::start_platform_login,
            commands::poll_platform_login,
            commands::resolve_media_preview,
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
