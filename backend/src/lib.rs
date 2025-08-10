pub mod export;
pub mod meta;
pub mod debugger;
pub mod search;
pub mod plugins;

use once_cell::sync::Lazy;
use plugins::{Plugin, wasm::WasmPlugin};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use tree_sitter::Tree;

/// Stored parse trees for opened documents.
static DOCUMENT_TREES: Lazy<Mutex<HashMap<String, Tree>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

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
/// executed in a sandboxed WebAssembly runtime using `wasmtime`.
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

/// Load a WebAssembly plugin using [`wasmtime`].
fn load_wasm(path: &Path) -> Option<Box<dyn Plugin>> {
    WasmPlugin::from_file(path).map(|p| Box::new(p) as Box<dyn Plugin>)
}
