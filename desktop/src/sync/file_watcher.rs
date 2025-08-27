use crate::components::file_manager::FileManagerPlugin;
use notify::{recommended_watcher, Event, EventKind, RecursiveMode, Watcher};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{mpsc::{channel, SyncSender}, Mutex};
use std::thread;

use multicode_core::parser::Lang;

use super::SyncMessage;

/// Watches a single file for modifications and forwards updates to the
/// synchronization engine via [`SyncMessage`]s.
///
/// The watcher integrates with the file manager through the
/// [`FileManagerPlugin`] trait, automatically starting whenever a file is
/// opened in the editor.
#[derive(Debug)]
pub struct FileWatcher {
    tx: SyncSender<Option<SyncMessage>>,
    watcher: Mutex<Option<notify::RecommendedWatcher>>,
}

impl FileWatcher {
    /// Creates a new file watcher plugin using the provided sender to forward
    /// [`SyncMessage`]s.
    pub fn new(tx: SyncSender<Option<SyncMessage>>) -> Self {
        Self {
            tx,
            watcher: Mutex::new(None),
        }
    }

    fn start_watching(&self, path: PathBuf) {
        let tx = self.tx.clone();
        let (evt_tx, evt_rx) = channel::<Event>();
        let mut watcher = recommended_watcher(move |res| {
            if let Ok(event) = res {
                let _ = evt_tx.send(event);
            }
        })
        .expect("file watcher");
        if watcher.watch(&path, RecursiveMode::NonRecursive).is_err() {
            return;
        }
        thread::spawn(move || {
            while let Ok(event) = evt_rx.recv() {
                match event.kind {
                    EventKind::Modify(_) => {
                        if let Ok(code) = fs::read_to_string(&path) {
                            if let Some(lang) = lang_from_path(&path) {
                                let _ = tx.send(Some(SyncMessage::TextChanged(code, lang)));
                            } else {
                                let _ = tx.send(Some(SyncMessage::ResetSync));
                            }
                        } else {
                            let _ = tx.send(Some(SyncMessage::ResetSync));
                        }
                    }
                    EventKind::Remove(_) => {
                        let _ = tx.send(Some(SyncMessage::ResetSync));
                    }
                    _ => {}
                }
            }
        });
        *self.watcher.lock().unwrap() = Some(watcher);
    }
}

impl FileManagerPlugin for FileWatcher {
    fn on_open(&self, path: &Path) {
        self.start_watching(path.to_path_buf());
    }
}

fn lang_from_path(path: &Path) -> Option<Lang> {
    match path.extension().and_then(|s| s.to_str())? {
        "rs" => Some(Lang::Rust),
        "py" => Some(Lang::Python),
        "js" => Some(Lang::JavaScript),
        "css" => Some(Lang::Css),
        "html" => Some(Lang::Html),
        "go" => Some(Lang::Go),
        "ts" => Some(Lang::TypeScript),
        "c" => Some(Lang::C),
        "cpp" | "c++" | "cc" => Some(Lang::Cpp),
        "java" => Some(Lang::Java),
        "cs" | "csharp" => Some(Lang::CSharp),
        _ => None,
    }
}
