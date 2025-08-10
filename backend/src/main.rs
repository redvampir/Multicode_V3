use std::sync::Mutex;
use std::collections::HashMap;
use std::cmp::Ordering;
mod meta;
mod parser;
mod i18n;
mod git;
use meta::{upsert, read_all, remove_all, VisualMeta, Translations};
use parser::{parse, parse_to_blocks, Lang};
use tauri::State;
use serde::Serialize;
use syn::{File, Item};

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
    translations: Translations,
    range: (usize, usize),
    x: f64,
    y: f64,
}

#[tauri::command]
fn normalize_kind(kind: &str) -> String {
    let k = kind.to_lowercase();
    if k.contains("function") {
        "Function".into()
    } else if k.contains("if") {
        "Condition".into()
    } else if k.contains("for") || k.contains("while") || k.contains("loop") {
        "Loop".into()
    } else if k.contains("identifier") || k.contains("variable") {
        "Variable".into()
    } else {
        kind.to_string()
    }
}

#[tauri::command]
fn parse_blocks(content: String, lang: String) -> Option<Vec<BlockInfo>> {
    let lang = to_lang(&lang)?;
    let tree = parse(&content, lang)?;
    let blocks = parse_to_blocks(&tree);
    let metas = read_all(&content);
    let map: HashMap<_, _> = metas.into_iter().map(|m| (m.id.clone(), m)).collect();
    Some(blocks.into_iter().map(|b| {
        let label = normalize_kind(&b.kind);
        let mut translations = i18n::lookup(&label).unwrap_or_else(|| Translations {
            ru: Some(label.clone()),
            en: Some(label.clone()),
            es: Some(label.clone()),
        });
        if let Some(meta) = map.get(&b.visual_id) {
            let t = &meta.translations;
            if let Some(ref v) = t.ru { translations.ru = Some(v.clone()); }
            if let Some(ref v) = t.en { translations.en = Some(v.clone()); }
            if let Some(ref v) = t.es { translations.es = Some(v.clone()); }
        }
        let pos = map.get(&b.visual_id);
        BlockInfo {
            visual_id: b.visual_id,
            kind: label,
            translations,
            range: (b.range.start, b.range.end),
            x: pos.map(|m| m.x).unwrap_or(0.0),
            y: pos.map(|m| m.y).unwrap_or(0.0),
        }
    }).collect())
}

fn regenerate_code(content: &str, lang: Lang, metas: &[VisualMeta]) -> Option<String> {
    match lang {
        Lang::Rust => regenerate_rust(content, metas),
        _ => Some(content.to_string()),
    }
}

fn regenerate_rust(content: &str, metas: &[VisualMeta]) -> Option<String> {
    let mut file: File = syn::parse_file(content).ok()?;
    let tree = parse(content, Lang::Rust)?;
    let blocks = parse_to_blocks(&tree);
    let map: HashMap<_, _> = blocks.into_iter().map(|b| (b.node_id, b.visual_id)).collect();

    let mut cursor = tree.root_node().walk();
    let mut roots = Vec::new();
    if cursor.goto_first_child() {
        loop {
            roots.push(cursor.node().id());
            if !cursor.goto_next_sibling() { break; }
        }
    }

    let mut items: Vec<(Item, (f64, f64))> = file.items.into_iter().zip(roots.into_iter()).map(|(it, id)| {
        let vid = map.get(&id).cloned().unwrap_or_default();
        let pos = metas.iter().find(|m| m.id == vid).map(|m| (m.y, m.x)).unwrap_or((0.0,0.0));
        (it, pos)
    }).collect();

    items.sort_by(|a, b| {
        a.1.0.partial_cmp(&b.1.0).unwrap_or(Ordering::Equal)
            .then_with(|| a.1.1.partial_cmp(&b.1.1).unwrap_or(Ordering::Equal))
    });

    file.items = items.into_iter().map(|(it, _)| it).collect();
    Some(prettyplease::unparse(&file))
}

#[tauri::command]
fn upsert_meta(content: String, meta: VisualMeta, lang: String) -> String {
    let mut metas = read_all(&content);
    metas.retain(|m| m.id != meta.id);
    metas.push(meta);

    let cleaned = remove_all(&content);
    let lang = to_lang(&lang).unwrap_or(Lang::Rust);
    let regenerated = regenerate_code(&cleaned, lang, &metas).unwrap_or(cleaned);

    metas.into_iter().fold(regenerated, |acc, m| upsert(&acc, &m))
}

#[tauri::command]
fn export_clean(path: String, state: State<EditorState>) -> Result<(), String> {
    let content = state.0.lock().unwrap().clone();
    let cleaned = remove_all(&content);
    std::fs::write(path, cleaned).map_err(|e| e.to_string())
}

#[tauri::command]
fn git_commit_cmd(message: String) -> Result<(), String> {
    git::commit(&message).map_err(|e| e.to_string())
}

#[tauri::command]
fn git_diff_cmd() -> Result<String, String> {
    git::diff().map_err(|e| e.to_string())
}

#[tauri::command]
fn git_branches_cmd() -> Result<Vec<String>, String> {
    git::branches().map_err(|e| e.to_string())
}

#[tauri::command]
fn git_log_cmd() -> Result<Vec<String>, String> {
    git::log().map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .manage(EditorState::default())
        .invoke_handler(tauri::generate_handler![
            save_state,
            load_state,
            parse_blocks,
            upsert_meta,
            export_clean,
            git_commit_cmd,
            git_diff_cmd,
            git_branches_cmd,
            git_log_cmd
        ])
        .run(tauri::generate_context!(
            "../frontend/src-tauri/tauri.conf.json"
        ))
        .expect("error while running tauri application");
}
