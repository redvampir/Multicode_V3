use std::sync::Mutex;
mod meta;
mod parser;
use meta::{insert, read, VisualMeta};
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

#[tauri::command]
fn insert_meta(content: String, meta: VisualMeta) -> String {
    insert(&content, &meta)
}

#[tauri::command]
fn read_meta(content: String) -> Option<VisualMeta> {
    read(&content)
}

fn main() {
    tauri::Builder::default()
        .manage(EditorState::default())
        .invoke_handler(tauri::generate_handler![
            save_state,
            load_state,
            insert_meta,
            read_meta
        ])
        .run(tauri::generate_context!(
            "../frontend/src-tauri/tauri.conf.json"
        ))
        .expect("error while running tauri application");
}
