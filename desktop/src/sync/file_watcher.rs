use crate::components::file_manager::FileManagerPlugin;
use globset::{Glob, GlobSet, GlobSetBuilder};
use notify::{recommended_watcher, Event, EventKind, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::sync::{
    mpsc::{channel, SyncSender},
    Arc, Mutex,
};
use std::thread;
use std::time::{Duration, Instant};

use multicode_core::parser::Lang;

use super::SyncMessage;

const IGNORE_PATTERNS: &[&str] = &["*~", "*.swp", "*.tmp", "#*#", ".git/*"];
const DEBOUNCE_DELAY: Duration = Duration::from_millis(100);

/// Watches files for modifications and forwards updates to the
/// synchronization engine via [`SyncMessage`]s.
#[derive(Debug)]
pub struct FileWatcher {
    tx: SyncSender<Option<SyncMessage>>,
    watchers: Mutex<HashMap<PathBuf, notify::RecommendedWatcher>>,
    last_events: Arc<Mutex<HashMap<PathBuf, Instant>>>,
    ignore: GlobSet,
}

impl FileWatcher {
    /// Creates a new file watcher plugin using the provided sender to forward
    /// [`SyncMessage`]s.
    pub fn new(tx: SyncSender<Option<SyncMessage>>) -> Self {
        let mut builder = GlobSetBuilder::new();
        for pat in IGNORE_PATTERNS {
            builder.add(Glob::new(pat).expect("invalid glob"));
        }
        let ignore = builder.build().expect("globset build");
        Self {
            tx,
            watchers: Mutex::new(HashMap::new()),
            last_events: Arc::new(Mutex::new(HashMap::new())),
            ignore,
        }
    }

    fn start_watching(&self, path: PathBuf) {
        if self.ignore.is_match(&path) {
            return;
        }
        let tx = self.tx.clone();
        let (evt_tx, evt_rx) = channel::<Event>();
        let mut watcher = match recommended_watcher(move |res| {
            if let Ok(event) = res {
                let _ = evt_tx.send(event);
            }
        }) {
            Ok(w) => w,
            Err(e) => {
                tracing::error!(error = ?e, "failed to create watcher");
                return;
            }
        };
        if let Err(e) = watcher.watch(&path, RecursiveMode::NonRecursive) {
            tracing::error!(?e, ?path, "failed to watch file");
            return;
        }
        let ignore = self.ignore.clone();
        let last_events = self.last_events.clone();
        thread::spawn(move || {
            while let Ok(event) = evt_rx.recv() {
                for p in event.paths {
                    if ignore.is_match(&p) {
                        continue;
                    }
                    let mut last = last_events.lock().unwrap();
                    let now = Instant::now();
                    if let Some(prev) = last.get(&p) {
                        if now.duration_since(*prev) < DEBOUNCE_DELAY {
                            continue;
                        }
                    }
                    last.insert(p.clone(), now);
                    drop(last);
                    match event.kind {
                        EventKind::Modify(_) => match read_file(&p) {
                            Ok(code) => {
                                if let Some(lang) = lang_from_path(&p) {
                                    let _ = tx.send(Some(SyncMessage::TextChanged(code, lang)));
                                } else {
                                    let _ = tx.send(Some(SyncMessage::ResetSync));
                                }
                            }
                            Err(e) => {
                                tracing::error!(?e, ?p, "failed to read file");
                                let _ = tx.send(Some(SyncMessage::ResetSync));
                            }
                        },
                        EventKind::Remove(_) => {
                            let _ = tx.send(Some(SyncMessage::ResetSync));
                        }
                        _ => {}
                    }
                }
            }
        });
        if let Ok(mut map) = self.watchers.lock() {
            map.insert(path, watcher);
        }
    }
}

impl FileManagerPlugin for FileWatcher {
    fn on_open(&self, path: &Path) {
        self.start_watching(path.to_path_buf());
    }
}

fn read_file(path: &Path) -> std::io::Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;
    Ok(buf)
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
