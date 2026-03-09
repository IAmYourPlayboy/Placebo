use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: String,
    pub name: String,
    pub video_url: String,
    pub participants: Vec<String>,
    pub is_playing: bool,
    pub current_timestamp: f64,
    pub video_source: String, // "youtube" | "twitch" | "url"
    pub is_live: bool,
    pub viewer_count: u32,
}

#[derive(Default)]
pub struct AppState {
    pub current_user_id: Mutex<String>,
}

// ─── Commands ─────────────────────────────────────────────────────────────────

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Привет, {}! Добро пожаловать в Placebo.", name)
}

#[tauri::command]
fn get_user_id(state: State<'_, AppState>) -> String {
    let id = state.current_user_id.lock().unwrap();
    if id.is_empty() {
        // Generate a random-ish ID until we have real auth
        format!("user_{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos())
    } else {
        id.clone()
    }
}

#[tauri::command]
fn get_public_rooms() -> Vec<Room> {
    // Stub data — will be replaced by WebSocket server calls
    vec![
        Room {
            id: "room_1".into(),
            name: "Концерт Шамана".into(),
            video_url: "https://youtube.com/watch?v=example1".into(),
            participants: vec!["user_a".into(), "user_b".into()],
            is_playing: true,
            current_timestamp: 142.5,
            video_source: "youtube".into(),
            is_live: true,
            viewer_count: 52,
        },
        Room {
            id: "room_2".into(),
            name: "Аниме-марафон".into(),
            video_url: "https://youtube.com/watch?v=example2".into(),
            participants: vec!["user_c".into()],
            is_playing: false,
            current_timestamp: 0.0,
            video_source: "youtube".into(),
            is_live: false,
            viewer_count: 38,
        },
    ]
}

#[tauri::command]
fn create_room(name: String, video_url: String, video_source: String) -> Room {
    Room {
        id: format!("room_{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()),
        name,
        video_url,
        participants: vec![],
        is_playing: false,
        current_timestamp: 0.0,
        video_source,
        is_live: false,
        viewer_count: 0,
    }
}

// ─── Entry ────────────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_user_id,
            get_public_rooms,
            create_room,
        ])
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                use tauri::Manager;
                app.get_webview_window("main")
                    .unwrap()
                    .open_devtools();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
