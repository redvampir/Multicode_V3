pub mod export;
pub mod meta;
pub mod debugger;
pub mod search;
pub mod plugins;

use plugins::Plugin;
use std::path::Path;

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

    if let Ok(entries) = std::fs::read_dir("plugins") {
        for entry in entries.flatten() {
            let path = entry.path();
            match path.extension().and_then(|e| e.to_str()) {
                Some("dll") | Some("so") | Some("dylib") => {
                    if let Some(p) = unsafe { load_dll(&path) } {
                        loaded.push(p);
                    }
                }
                Some("wasm") => {
                    if let Some(p) = load_wasm(&path) {
                        loaded.push(p);
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
