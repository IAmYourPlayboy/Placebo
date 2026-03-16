use crate::db;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub async fn get_cameras(state: State<'_, AppState>) -> Result<Vec<db::camera::Camera>, String> {
    db::camera::get_all(&state.db).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_camera_by_id(
    state: State<'_, AppState>,
    id: String,
) -> Result<Option<db::camera::Camera>, String> {
    db::camera::get_by_id(&state.db, &id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_cameras_by_city(
    state: State<'_, AppState>,
    city: String,
) -> Result<Vec<db::camera::Camera>, String> {
    db::camera::get_by_city(&state.db, &city)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_cameras(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<db::camera::Camera>, String> {
    db::camera::search(&state.db, &query)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_nearby_cameras(
    state: State<'_, AppState>,
    lat: f64,
    lng: f64,
    radius_km: f64,
) -> Result<Vec<db::camera::Camera>, String> {
    db::camera::get_nearby(&state.db, lat, lng, radius_km)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_camera_categories(
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    db::camera::get_categories(&state.db)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_cameras_count(
    state: State<'_, AppState>,
) -> Result<i64, String> {
    db::camera::get_count(&state.db)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn seed_cameras(state: State<'_, AppState>) -> Result<usize, String> {
    db::seed::seed_cameras(&state.db)
        .await
        .map_err(|e| e.to_string())
}
