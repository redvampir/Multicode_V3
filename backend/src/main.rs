use serde::Serialize;
use std::sync::Mutex;
use tauri::State;

#[derive(Default)]
struct EditorState(Mutex<String>);

#[tauri::command]
fn save_state(state: State<EditorState>, content: String) {
    *state.0.lock().unwrap() = content;
}

#[tauri::command]
fn load_state(state: State<EditorState>) -> String {
    state.0.lock().unwrap().clone()
}

fn main() {
    tauri::Builder::default()
        .manage(EditorState::default())
        .invoke_handler(tauri::generate_handler![save_state, load_state])
        .run(tauri::generate_context!("../frontend/src-tauri/tauri.conf.json"))
        .expect("error while running tauri application");
}
