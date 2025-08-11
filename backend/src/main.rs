use std::collections::HashMap;
use std::sync::Mutex;
mod config;
mod debugger;
mod export;
mod git;
mod meta;
mod parser;
mod plugins;
mod server;
pub use crate::BlockInfo;
use crate::blocks::{parse_blocks, to_lang, upsert_meta};
use clap::{Parser, Subcommand};
use debugger::{debug_break, debug_run, debug_step};
use export::prepare_for_export;
use meta::{fix_all, read_all, remove_all, upsert, AiNote, VisualMeta};
use parser::{parse, parse_to_blocks, Lang};
use tauri::State;

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

#[cfg_attr(not(test), tauri::command)]
fn save_state(state: State<EditorState>, content: String) {
    *state.0.lock().unwrap() = content;
}

#[cfg_attr(not(test), tauri::command)]
fn load_state(state: State<EditorState>) -> String {
    state.0.lock().unwrap().clone()
}

#[cfg_attr(not(test), tauri::command)]
fn suggest_ai_note(_content: String, _lang: String) -> AiNote {
    AiNote {
        description: Some("Not implemented".into()),
        hints: Vec::new(),
    }
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
    use backend::blocks::{parse_blocks, upsert_meta};
    use chrono::Utc;

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
        let metas = meta::read_all(&updated);
        assert_eq!(metas.len(), 1);
        assert_eq!(
            metas[0].translations.get("en").map(|s| s.as_str()),
            Some("Main")
        );
    }

    #[test]
    fn export_removes_metadata() {
        let src = format!("<!-- @VISUAL_META {{\"id\":\"1\"}} -->\nfn main() {{}}\n");
        let cleaned = export::prepare_for_export(&src, true);
        assert!(!cleaned.contains("@VISUAL_META"));
        assert!(cleaned.contains("fn main"));
    }
}
