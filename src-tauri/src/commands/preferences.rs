use crate::{db, AppState};
use tauri::State;

#[tauri::command]
pub async fn prefs_get(
    state: State<'_, AppState>,
    key: String,
) -> Result<Option<String>, String> {
    db::preferences::get(&state.db, &key)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn prefs_set(
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> Result<(), String> {
    db::preferences::set(&state.db, &key, &value)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn prefs_all(
    state: State<'_, AppState>,
) -> Result<Vec<db::preferences::Preference>, String> {
    db::preferences::all(&state.db)
        .await
        .map_err(|e| e.to_string())
}
