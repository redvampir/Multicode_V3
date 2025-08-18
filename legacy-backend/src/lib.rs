pub mod blocks;
pub mod config;
pub mod debugger;
pub mod export;
pub mod git;
mod i18n;
pub mod meta;
pub mod parser;
pub mod plugins;
pub mod search;
pub mod server;
pub mod viz_lint;

pub use blocks::{parse_blocks, upsert_meta};

use crate::meta::AiNote;
use once_cell::sync::Lazy;
use plugins::{Plugin, WasmPlugin};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use tracing::error;
use tree_sitter::Tree;

/// Stored parse trees for opened documents.
static DOCUMENT_TREES: Lazy<Mutex<HashMap<String, Tree>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Cached parsed blocks keyed by path or content hash.
static BLOCK_CACHE: Lazy<Mutex<HashMap<String, (String, Vec<BlockInfo>)>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct BlockInfo {
    pub visual_id: String,
    #[serde(default)]
    pub node_id: Option<u32>,
    pub kind: String,
    pub translations: HashMap<String, String>,
    pub range: (usize, usize),
    #[serde(default)]
    pub anchors: Vec<(usize, usize)>,
    pub x: f64,
    pub y: f64,
    pub ai: Option<AiNote>,
    #[serde(default)]
    pub links: Vec<String>,
}

/// Retrieve the last parsed [`Tree`] for the given document identifier.
pub fn get_document_tree(id: &str) -> Option<Tree> {
    match DOCUMENT_TREES.lock() {
        Ok(trees) => trees.get(id).cloned(),
        Err(e) => {
            error!("failed to lock document trees: {e}");
            None
        }
    }
}

/// Update the stored [`Tree`] for the given document identifier.
pub fn update_document_tree(id: String, tree: Tree) {
    match DOCUMENT_TREES.lock() {
        Ok(mut trees) => {
            trees.insert(id, tree);
        }
        Err(e) => error!("failed to lock document trees for update: {e}"),
    }
}

/// Retrieve cached blocks if the content matches.
pub fn get_cached_blocks(key: &str, content: &str) -> Option<Vec<BlockInfo>> {
    match BLOCK_CACHE.lock() {
        Ok(cache) => {
            if let Some((cached_content, blocks)) = cache.get(key) {
                if cached_content == content {
                    return Some(blocks.clone());
                }
            }
            None
        }
        Err(e) => {
            error!("failed to lock block cache: {e}");
            None
        }
    }
}

/// Update the block cache for the given key.
pub fn update_block_cache(key: String, content: String, blocks: Vec<BlockInfo>) {
    match BLOCK_CACHE.lock() {
        Ok(mut cache) => {
            cache.insert(key, (content, blocks));
        }
        Err(e) => error!("failed to lock block cache for update: {e}"),
    }
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
fn load_wasm(path: &Path) -> Option<Box<dyn Plugin>> {
    WasmPlugin::from_file(path).map(|p| Box::new(p) as Box<dyn Plugin>)
}

const SETTINGS_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../frontend/settings.json");

#[derive(Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub enabled: bool,
}

static ACTIVE_PLUGINS: Lazy<Mutex<Vec<Box<dyn Plugin>>>> = Lazy::new(|| Mutex::new(Vec::new()));
static PLUGIN_INFOS: Lazy<Mutex<Vec<PluginInfo>>> = Lazy::new(|| Mutex::new(Vec::new()));

fn read_plugin_settings() -> HashMap<String, bool> {
    let mut map = HashMap::new();
    if let Ok(data) = std::fs::read_to_string(SETTINGS_PATH) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&data) {
            if let Some(obj) = json.get("plugins").and_then(|p| p.as_object()) {
                for (k, v) in obj {
                    if let Some(b) = v.as_bool() {
                        map.insert(k.clone(), b);
                    }
                }
            }
        }
    }
    map
}

fn write_plugin_settings(settings: &HashMap<String, bool>) -> std::io::Result<()> {
    use std::fs;
    let mut json: serde_json::Value = serde_json::from_str(&fs::read_to_string(SETTINGS_PATH)?)?;
    if let Some(obj) = json.as_object_mut() {
        obj.insert(
            "plugins".to_string(),
            serde_json::to_value(settings).unwrap_or_default(),
        );
    }
    fs::write(SETTINGS_PATH, serde_json::to_string_pretty(&json)?)
}

pub fn reload_plugins_state() {
    let settings = read_plugin_settings();
    let mut infos = Vec::new();
    let mut active = Vec::new();
    for plugin in load_plugins() {
        let name = plugin.name().to_string();
        let info = PluginInfo {
            name: name.clone(),
            version: plugin.version().to_string(),
            enabled: *settings.get(&name).unwrap_or(&true),
        };
        if info.enabled {
            active.push(plugin);
        }
        infos.push(info);
    }
    *ACTIVE_PLUGINS.lock().unwrap() = active;
    *PLUGIN_INFOS.lock().unwrap() = infos;
}

pub fn get_plugins_info() -> Vec<PluginInfo> {
    PLUGIN_INFOS.lock().unwrap().clone()
}

pub fn set_plugin_enabled(name: String, enabled: bool) -> std::io::Result<()> {
    let mut settings = read_plugin_settings();
    settings.insert(name, enabled);
    write_plugin_settings(&settings)?;
    reload_plugins_state();
    Ok(())
}
