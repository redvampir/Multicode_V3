use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Mutex;
mod config;
mod debugger;
mod export;
mod git;
mod i18n;
mod meta;
mod parser;
mod plugins;
mod server;
use backend::{get_document_tree, update_document_tree};
use debugger::{debug_break, debug_run, debug_step};
use export::prepare_for_export;
use meta::{read_all, remove_all, upsert, AiNote, Translations, VisualMeta};
use parser::{parse, parse_to_blocks, Lang};
use serde::Serialize;
use syn::{File, Item};
use tauri::State;

#[derive(Default)]
struct EditorState(Mutex<String>);

#[cfg_attr(not(test), tauri::command)]
fn save_state(state: State<EditorState>, content: String) {
    *state.0.lock().unwrap() = content;
}

#[cfg_attr(not(test), tauri::command)]
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
pub struct BlockInfo {
    visual_id: String,
    kind: String,
    translations: Translations,
    range: (usize, usize),
    x: f64,
    y: f64,
    ai: Option<AiNote>,
}

#[cfg_attr(not(test), tauri::command)]
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

#[cfg_attr(not(test), tauri::command)]
pub fn parse_blocks(content: String, lang: String) -> Option<Vec<BlockInfo>> {
    let lang = to_lang(&lang)?;
    let old = get_document_tree("current");
    let tree = parse(&content, lang, old.as_ref())?;
    update_document_tree("current".to_string(), tree.clone());
    let blocks = parse_to_blocks(&tree);
    let metas = read_all(&content);
    let map: HashMap<_, _> = metas.into_iter().map(|m| (m.id.clone(), m)).collect();
    Some(
        blocks
            .into_iter()
            .map(|b| {
                let label = normalize_kind(&b.kind);
                let mut translations = i18n::lookup(&label).unwrap_or_else(|| Translations {
                    ru: Some(label.clone()),
                    en: Some(label.clone()),
                    es: Some(label.clone()),
                });
                if let Some(meta) = map.get(&b.visual_id) {
                    let t = &meta.translations;
                    if let Some(ref v) = t.ru {
                        translations.ru = Some(v.clone());
                    }
                    if let Some(ref v) = t.en {
                        translations.en = Some(v.clone());
                    }
                    if let Some(ref v) = t.es {
                        translations.es = Some(v.clone());
                    }
                }
                let pos = map.get(&b.visual_id);
                BlockInfo {
                    visual_id: b.visual_id,
                    kind: label,
                    translations,
                    range: (b.range.start, b.range.end),
                    x: pos.map(|m| m.x).unwrap_or(0.0),
                    y: pos.map(|m| m.y).unwrap_or(0.0),
                    ai: pos.and_then(|m| m.ai.clone()),
                }
            })
            .collect(),
    )
}

#[cfg_attr(not(test), tauri::command)]
fn suggest_ai_note(_content: String, _lang: String) -> AiNote {
    AiNote {
        description: Some("Not implemented".into()),
        hints: Vec::new(),
    }
}

fn regenerate_code(content: &str, lang: Lang, metas: &[VisualMeta]) -> Option<String> {
    match lang {
        Lang::Rust => regenerate_rust(content, metas),
        _ => Some(content.to_string()),
    }
}

fn regenerate_rust(content: &str, metas: &[VisualMeta]) -> Option<String> {
    let mut file: File = syn::parse_file(content).ok()?;
    let tree = parse(content, Lang::Rust, None)?;
    let blocks = parse_to_blocks(&tree);
    let map: HashMap<_, _> = blocks
        .into_iter()
        .map(|b| (b.node_id, b.visual_id))
        .collect();

    let mut cursor = tree.root_node().walk();
    let mut roots = Vec::new();
    if cursor.goto_first_child() {
        loop {
            roots.push(cursor.node().id());
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }

    let mut items: Vec<(Item, (f64, f64))> = file
        .items
        .into_iter()
        .zip(roots.into_iter())
        .map(|(it, id)| {
            let vid = map.get(&id).cloned().unwrap_or_default();
            let pos = metas
                .iter()
                .find(|m| m.id == vid)
                .map(|m| (m.y, m.x))
                .unwrap_or((0.0, 0.0));
            (it, pos)
        })
        .collect();

    items.sort_by(|a, b| {
        a.1 .0
            .partial_cmp(&b.1 .0)
            .unwrap_or(Ordering::Equal)
            .then_with(|| a.1 .1.partial_cmp(&b.1 .1).unwrap_or(Ordering::Equal))
    });

    file.items = items.into_iter().map(|(it, _)| it).collect();
    Some(prettyplease::unparse(&file))
}

#[cfg_attr(not(test), tauri::command)]
pub fn upsert_meta(content: String, mut meta: VisualMeta, lang: String) -> String {
    let mut metas = read_all(&content);
    if let Some(existing) = metas.iter().find(|m| m.id == meta.id) {
        if meta.translations.ru.is_none()
            && meta.translations.en.is_none()
            && meta.translations.es.is_none()
        {
            meta.translations = existing.translations.clone();
        }
        if meta.ai.is_none() {
            meta.ai = existing.ai.clone();
        }
    }
    metas.retain(|m| m.id != meta.id);
    metas.push(meta);

    let cleaned = remove_all(&content);
    let lang = to_lang(&lang).unwrap_or(Lang::Rust);
    let regenerated = regenerate_code(&cleaned, lang, &metas).unwrap_or(cleaned);

    metas
        .into_iter()
        .fold(regenerated, |acc, m| upsert(&acc, &m))
}

#[cfg_attr(not(test), tauri::command)]
fn export_file(path: String, strip_meta: bool, state: State<EditorState>) -> Result<(), String> {
    let content = state.0.lock().unwrap().clone();
    let out = prepare_for_export(&content, strip_meta);
    std::fs::write(path, out).map_err(|e| e.to_string())
}

#[cfg_attr(not(test), tauri::command)]
fn git_commit_cmd(message: String) -> Result<(), String> {
    git::commit(&message).map_err(|e| e.to_string())
}

#[cfg_attr(not(test), tauri::command)]
fn git_diff_cmd() -> Result<String, String> {
    git::diff().map_err(|e| e.to_string())
}

#[cfg_attr(not(test), tauri::command)]
fn git_branches_cmd() -> Result<Vec<String>, String> {
    git::branches().map_err(|e| e.to_string())
}

#[cfg_attr(not(test), tauri::command)]
fn git_log_cmd() -> Result<Vec<String>, String> {
    git::log().map_err(|e| e.to_string())
}

#[cfg(not(test))]
fn main() {
    let log_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../logs");
    std::fs::create_dir_all(&log_dir).expect("create logs directory");
    let file_appender = tracing_appender::rolling::daily(log_dir, "backend.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .init();
    tauri::async_runtime::spawn(async {
        server::run().await;
    });
    tauri::Builder::default()
        .manage(EditorState::default())
        .invoke_handler(tauri::generate_handler![
            save_state,
            load_state,
            parse_blocks,
            suggest_ai_note,
            upsert_meta,
            export_file,
            git_commit_cmd,
            git_diff_cmd,
            git_branches_cmd,
            git_log_cmd,
            debug_run,
            debug_step,
            debug_break
        ])
        .run(tauri::generate_context!(
            "../frontend/src-tauri/tauri.conf.json"
        ))
        .expect("error while running tauri application");
}

#[cfg(test)]
fn main() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_source_into_blockinfo() {
        let src = "fn main() {}".to_string();
        let blocks = parse_blocks(src, "rust".into()).expect("parse");
        assert!(!blocks.is_empty());
        assert!(blocks.iter().any(|b| b.kind == "Function"));
    }

    #[test]
    fn upsert_meta_synchronizes_data() {
        let src = "fn main() {}".to_string();
        let meta = VisualMeta {
            id: "0".into(),
            x: 1.0,
            y: 2.0,
            translations: Translations {
                en: Some("Main".into()),
                ..Default::default()
            },
            ai: None,
        };
        let updated = upsert_meta(src, meta.clone(), "rust".into());
        assert!(updated.contains("@VISUAL_META"));
        let metas = meta::read_all(&updated);
        assert_eq!(metas.len(), 1);
        assert_eq!(metas[0].translations.en.as_deref(), Some("Main"));
    }

    #[test]
    fn export_removes_metadata() {
        let src = format!("<!-- @VISUAL_META {{\"id\":\"1\"}} -->\nfn main() {{}}\n");
        let cleaned = export::prepare_for_export(&src, true);
        assert!(!cleaned.contains("@VISUAL_META"));
        assert!(cleaned.contains("fn main"));
    }
}
