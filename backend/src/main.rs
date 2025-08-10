use std::sync::Mutex;
use std::collections::HashMap;
mod meta;
mod parser;
use meta::{upsert, read_all, VisualMeta};
use parser::{parse, parse_to_blocks, Lang};
use tauri::State;
use serde::Serialize;

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

fn to_lang(s: &str) -> Option<Lang> {
    match s.to_lowercase().as_str() {
        "rust" => Some(Lang::Rust),
        "python" => Some(Lang::Python),
        "javascript" => Some(Lang::JavaScript),
        "css" => Some(Lang::Css),
        "html" => Some(Lang::Html),
        _ => None,
    }
}

#[derive(Serialize)]
struct BlockInfo {
    visual_id: String,
    kind: String,
    range: (usize, usize),
    x: f64,
    y: f64,
}

#[tauri::command]
fn parse_blocks(content: String, lang: String) -> Option<Vec<BlockInfo>> {
    let lang = to_lang(&lang)?;
    let tree = parse(&content, lang)?;
    let blocks = parse_to_blocks(&tree);
    let metas = read_all(&content);
    let map: HashMap<_, _> = metas.into_iter().map(|m| (m.id.clone(), m)).collect();
    Some(blocks.into_iter().map(|b| {
        let pos = map.get(&b.visual_id);
        BlockInfo {
            visual_id: b.visual_id,
            kind: b.kind,
            range: (b.range.start, b.range.end),
            x: pos.map(|m| m.x).unwrap_or(0.0),
            y: pos.map(|m| m.y).unwrap_or(0.0),
        }
    }).collect())
}

#[tauri::command]
fn upsert_meta(content: String, meta: VisualMeta) -> String {
    upsert(&content, &meta)
}

fn main() {
    tauri::Builder::default()
        .manage(EditorState::default())
        .invoke_handler(tauri::generate_handler![
            save_state,
            load_state,
            parse_blocks,
            upsert_meta
        ])
        .run(tauri::generate_context!(
            "../frontend/src-tauri/tauri.conf.json"
        ))
        .expect("error while running tauri application");
}
