use std::sync::Mutex;

#[cfg(not(test))]
use backend::blocks::{__cmd__parse_blocks, __cmd__upsert_meta};
#[cfg(not(test))]
use backend::blocks::{parse_blocks, to_lang, upsert_meta};
#[cfg(not(test))]
use backend::debugger::{__cmd__debug_break, __cmd__debug_run, __cmd__debug_step};
#[cfg(not(test))]
use backend::debugger::{debug_break, debug_run, debug_step};
use backend::export::prepare_for_export;
#[cfg(not(test))]
use backend::git;
use backend::meta::read_all;
#[cfg(not(test))]
use backend::meta::{fix_all, remove_all, AiNote};
#[cfg(not(test))]
use backend::parser::{parse, parse_to_blocks};
#[cfg(not(test))]
use backend::server;
pub use backend::BlockInfo;
use clap::{Parser, Subcommand};
#[cfg(not(test))]
use tauri::State;

#[cfg(not(test))]
#[derive(Default)]
struct EditorState(Mutex<String>);

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse a source file and output block information as JSON
    Parse {
        /// Path to the source file
        path: String,
        /// Language of the source file
        #[arg(long)]
        lang: String,
    },
    /// Export a file, optionally stripping metadata comments
    Export {
        /// Input file path
        input: String,
        /// Output file path
        output: String,
        /// Remove @VISUAL_META comments
        #[arg(long)]
        strip_meta: bool,
    },
    /// Work with metadata comments
    Meta {
        #[command(subcommand)]
        command: MetaCommands,
    },
}

#[derive(Subcommand)]
enum MetaCommands {
    /// Show all metadata comments from a file as JSON
    List {
        /// Path to the source file
        path: String,
    },
    /// Fix metadata issues like duplicate identifiers
    Fix {
        /// Path to the source file
        path: String,
    },
    /// Remove all metadata comments from a file
    Remove {
        /// Path to the source file
        path: String,
    },
}

fn save_state_inner(state: &Mutex<String>, content: String) -> Result<(), String> {
    match state.lock() {
        Ok(mut guard) => {
            *guard = content;
            Ok(())
        }
        Err(e) => Err(e.to_string()),
    }
}

#[cfg_attr(not(test), tauri::command)]
#[cfg(not(test))]
fn save_state(state: State<EditorState>, content: String) -> Result<(), String> {
    save_state_inner(&state.0, content)
}

fn load_state_inner(state: &Mutex<String>) -> Result<String, String> {
    match state.lock() {
        Ok(guard) => Ok(guard.clone()),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg_attr(not(test), tauri::command)]
#[cfg(not(test))]
fn load_state(state: State<EditorState>) -> Result<String, String> {
    load_state_inner(&state.0)
}

#[cfg_attr(not(test), tauri::command)]
#[cfg(not(test))]
fn suggest_ai_note(_content: String, _lang: String) -> AiNote {
    AiNote {
        description: Some("Not implemented".into()),
        hints: Vec::new(),
    }
}

fn export_file_inner(path: String, strip_meta: bool, state: &Mutex<String>) -> Result<(), String> {
    let content = match state.lock() {
        Ok(guard) => guard.clone(),
        Err(e) => return Err(e.to_string()),
    };
    let out = prepare_for_export(&content, strip_meta);
    std::fs::write(path, out).map_err(|e| e.to_string())
}

#[cfg_attr(not(test), tauri::command)]
#[cfg(not(test))]
fn export_file(path: String, strip_meta: bool, state: State<EditorState>) -> Result<(), String> {
    export_file_inner(path, strip_meta, &state.0)
}

#[cfg_attr(not(test), tauri::command)]
#[cfg(not(test))]
fn git_commit_cmd(message: String) -> Result<(), String> {
    git::commit(&message).map_err(|e| e.to_string())
}

#[cfg_attr(not(test), tauri::command)]
#[cfg(not(test))]
fn git_diff_cmd() -> Result<String, String> {
    git::diff().map_err(|e| e.to_string())
}

#[cfg_attr(not(test), tauri::command)]
#[cfg(not(test))]
fn git_branches_cmd() -> Result<Vec<String>, String> {
    git::branches().map_err(|e| e.to_string())
}

#[cfg_attr(not(test), tauri::command)]
#[cfg(not(test))]
fn git_log_cmd() -> Result<Vec<String>, String> {
    git::log().map_err(|e| e.to_string())
}

#[cfg(not(test))]
fn main() {
    let cli = Cli::parse();
    if let Some(command) = cli.command {
        match command {
            Commands::Parse { path, lang } => {
                let content = std::fs::read_to_string(path).expect("read file");
                let lang = to_lang(&lang).expect("unknown language");
                let tree = parse(&content, lang, None).expect("parse");
                let blocks = parse_to_blocks(&tree);
                println!(
                    "{}",
                    serde_json::to_string_pretty(&blocks).expect("serialize blocks")
                );
            }
            Commands::Export {
                input,
                output,
                strip_meta,
            } => {
                let content = std::fs::read_to_string(&input).expect("read file");
                let out = prepare_for_export(&content, strip_meta);
                std::fs::write(output, out).expect("write file");
            }
            Commands::Meta { command } => match command {
                MetaCommands::List { path } => {
                    let content = std::fs::read_to_string(path).expect("read file");
                    let metas = read_all(&content);
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&metas).expect("serialize metadata")
                    );
                }
                MetaCommands::Fix { path } => {
                    let content = std::fs::read_to_string(&path).expect("read file");
                    let fixed = fix_all(&content);
                    std::fs::write(path, fixed).expect("write file");
                }
                MetaCommands::Remove { path } => {
                    let content = std::fs::read_to_string(&path).expect("read file");
                    let cleaned = remove_all(&content);
                    std::fs::write(path, cleaned).expect("write file");
                }
            },
        }
        return;
    }

    let log_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../logs");
    std::fs::create_dir_all(&log_dir).expect("create logs directory");
    let file_appender = tracing_appender::rolling::daily(log_dir, "backend.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .init();
    tauri::async_runtime::spawn(async {
        if let Err(e) = server::run().await {
            tracing::error!("server error: {e}");
        }
    });
    tauri::Builder::<tauri::Wry>::default()
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
    use backend::blocks::{parse_blocks, upsert_meta};
    use backend::meta::VisualMeta;
    use chrono::Utc;
    use std::collections::HashMap;

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
            version: 1,
            id: "0".into(),
            x: 1.0,
            y: 2.0,
            tags: vec![],
            links: vec![],
            extends: None,
            origin: None,
            translations: {
                let mut m = HashMap::new();
                m.insert("en".into(), "Main".into());
                m
            },
            ai: None,
            extras: None,
            updated_at: Utc::now(),
        };
        let updated = upsert_meta(src, meta.clone(), "rust".into());
        assert!(updated.contains("@VISUAL_META"));
        let metas = read_all(&updated);
        assert_eq!(metas.len(), 1);
        assert_eq!(
            metas[0].translations.get("en").map(|s| s.as_str()),
            Some("Main")
        );
    }

    #[test]
    fn export_removes_metadata() {
        let src = format!("<!-- @VISUAL_META {{\"id\":\"1\"}} -->\nfn main() {{}}\n");
        let cleaned = prepare_for_export(&src, true);
        assert!(!cleaned.contains("@VISUAL_META"));
        assert!(cleaned.contains("fn main"));
    }
    #[test]
    fn state_lock_poisoned_returns_err() {
        use std::sync::{Arc, Mutex};
        let state = Arc::new(Mutex::new(String::new()));
        let poisoned = Arc::clone(&state);
        let _ = std::thread::spawn(move || {
            let _lock = poisoned.lock().unwrap();
            panic!("poison");
        })
        .join();
        let path = std::env::temp_dir().join("export.txt");
        assert!(save_state_inner(state.as_ref(), "data".into()).is_err());
        assert!(load_state_inner(state.as_ref()).is_err());
        assert!(
            export_file_inner(path.to_string_lossy().to_string(), false, state.as_ref()).is_err()
        );
    }
}
