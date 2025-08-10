pub mod debugger;
pub mod export;
pub mod meta;
pub mod plugins;
pub mod search;

use crate::meta::{AiNote, Translations};
use once_cell::sync::Lazy;
use plugins::Plugin;
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use tree_sitter::Tree;

/// Stored parse trees for opened documents.
static DOCUMENT_TREES: Lazy<Mutex<HashMap<String, Tree>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Cached blocks for documents identified by path or hash.
static BLOCK_CACHE: Lazy<Mutex<HashMap<String, (String, Vec<BlockInfo>)>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Clone, Serialize)]
pub struct BlockInfo {
    pub visual_id: String,
    pub kind: String,
    pub translations: Translations,
    pub range: (usize, usize),
    pub x: f64,
    pub y: f64,
    pub ai: Option<AiNote>,
}

/// Retrieve cached [`BlockInfo`] entries if the content matches.
pub fn get_cached_blocks(id: &str, content: &str) -> Option<Vec<BlockInfo>> {
    let cache = BLOCK_CACHE.lock().unwrap();
    cache
        .get(id)
        .and_then(|(stored, blocks)| (stored == content).then(|| blocks.clone()))
}

/// Update the cache for the given document identifier.
pub fn update_cached_blocks(id: String, content: String, blocks: Vec<BlockInfo>) {
    BLOCK_CACHE.lock().unwrap().insert(id, (content, blocks));
}

/// Retrieve the last parsed [`Tree`] for the given document identifier.
pub fn get_document_tree(id: &str) -> Option<Tree> {
    DOCUMENT_TREES.lock().unwrap().get(id).cloned()
}

/// Update the stored [`Tree`] for the given document identifier.
pub fn update_document_tree(id: String, tree: Tree) {
    DOCUMENT_TREES.lock().unwrap().insert(id, tree);
}

/// Load all backend plugins from the `plugins/` directory.
///
/// Each file inside the directory is inspected and depending on the
/// extension the corresponding loader is invoked.  Native dynamic
/// libraries (`.dll`, `.so`, `.dylib`) are loaded via [`libloading`]
/// and must export a `create_plugin` function returning a boxed
/// implementation of [`Plugin`].  WebAssembly modules (`.wasm`) are
/// currently ignored but the function is prepared for future support.
pub fn load_plugins() -> Vec<Box<dyn Plugin>> {
    let mut loaded: Vec<Box<dyn Plugin>> = Vec::new();
    let expected = env!("CARGO_PKG_VERSION");

    if let Ok(entries) = std::fs::read_dir("plugins") {
        for entry in entries.flatten() {
            let path = entry.path();
            match path.extension().and_then(|e| e.to_str()) {
                Some("dll") | Some("so") | Some("dylib") => {
                    if let Some(p) = unsafe { load_dll(&path) } {
                        if p.version() == expected {
                            loaded.push(p);
                        } else {
                            eprintln!(
                                "Skipping plugin {} due to version mismatch: {} != {}",
                                p.name(),
                                p.version(),
                                expected
                            );
                        }
                    }
                }
                Some("wasm") => {
                    if let Some(p) = load_wasm(&path) {
                        if p.version() == expected {
                            loaded.push(p);
                        } else {
                            eprintln!(
                                "Skipping plugin {} due to version mismatch: {} != {}",
                                p.name(),
                                p.version(),
                                expected
                            );
                        }
                    }
                }
                _ => {}
            }
        }
    }

    loaded
}

/// Load a native plugin using the `libloading` crate.  The dynamic
/// library must expose a `create_plugin` function with the signature
/// `fn() -> Box<dyn Plugin>`.
unsafe fn load_dll(path: &Path) -> Option<Box<dyn Plugin>> {
    use libloading::{Library, Symbol};

    type Constructor = unsafe fn() -> Box<dyn Plugin>;

    let lib = Library::new(path).ok()?;
    let constructor: Symbol<Constructor> = lib.get(b"create_plugin").ok()?;
    let plugin = constructor();

    // The library must stay loaded for the lifetime of the plugin.
    // Leak the library to keep it in memory.
    std::mem::forget(lib);
    Some(plugin)
}

/// Load a WebAssembly plugin.  This is currently a stub that returns
/// `None`, but the function is provided to make extending the loader in
/// the future straightforward.
fn load_wasm(_path: &Path) -> Option<Box<dyn Plugin>> {
    None
}
