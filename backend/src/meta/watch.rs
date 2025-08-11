use crate::{blocks::parse_blocks, BlockInfo};
use notify::{recommended_watcher, Event, EventKind, RecursiveMode, Watcher};
use std::{env, fs, path::PathBuf, sync::mpsc::channel, thread};
use tokio::sync::broadcast::Sender;

/// Spawn a background thread watching the current directory for changes to
/// source files and `.meta.json` files.  When a file is written the
/// corresponding source is parsed and the resulting blocks are sent to the
/// provided broadcast channel as a JSON string.
pub fn spawn(tx: Sender<String>) {
    thread::spawn(move || {
        let (fs_tx, fs_rx) = channel::<Event>();
        let mut watcher = recommended_watcher(move |res| {
            if let Ok(event) = res {
                let _ = fs_tx.send(event);
            }
        })
        .expect("watcher");
        let path = env::current_dir().expect("cwd");
        watcher
            .watch(&path, RecursiveMode::Recursive)
            .expect("watch");
        while let Ok(event) = fs_rx.recv() {
            if let EventKind::Modify(_) = event.kind {
                if let Some(path) = event.paths.first() {
                    if let Some(src_path) = source_path(path) {
                        if let Some(lang) = language_from_path(&src_path) {
                            if let Ok(content) = fs::read_to_string(&src_path) {
                                if let Some(blocks) = parse_blocks(content, lang.into()) {
                                    if let Ok(json) = serde_json::to_string(&blocks) {
                                        let _ = tx.send(json);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    });
}

fn language_from_path(path: &PathBuf) -> Option<&'static str> {
    match path.extension().and_then(|s| s.to_str()) {
        Some("rs") => Some("rust"),
        Some("py") => Some("python"),
        Some("js") => Some("javascript"),
        Some("css") => Some("css"),
        Some("html") => Some("html"),
        _ => None,
    }
}

fn source_path(path: &PathBuf) -> Option<PathBuf> {
    if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
        if name.ends_with(".meta.json") {
            let mut s = path.to_string_lossy().to_string();
            s.truncate(s.len() - ".meta.json".len());
            return Some(PathBuf::from(s));
        }
    }
    Some(path.clone())
}
