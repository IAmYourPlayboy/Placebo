mod commands;
mod db;

use sqlx::sqlite::SqlitePool;

pub struct AppState {
    pub db: SqlitePool,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            use tauri::Manager;

            let data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to get app data dir");
            std::fs::create_dir_all(&data_dir)?;

            let pool = tauri::async_runtime::block_on(db::init(&data_dir))
                .expect("failed to init database");

            app.manage(AppState { db: pool });

            #[cfg(debug_assertions)]
            {
                if let Some(window) = app.get_webview_window("main") {
                    window.open_devtools();
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::camera::get_cameras,
            commands::camera::get_camera_by_id,
            commands::camera::get_cameras_by_city,
            commands::camera::search_cameras,
            commands::camera::get_nearby_cameras,
            commands::camera::get_camera_categories,
            commands::camera::get_cameras_count,
            commands::camera::seed_cameras,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
