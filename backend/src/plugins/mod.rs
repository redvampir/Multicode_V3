use serde::{Deserialize, Serialize};
use std::path::Path;
use wasmtime::{Engine, Instance, Memory, Module, Store};

/// Description of a block type provided by a plugin.
///
/// Backend uses this information to tell the frontend which visual
/// components should be loaded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDescriptor {
    /// Identifier of the block kind.
    pub kind: String,
    /// Optional human‑readable label for the block.
    pub label: Option<String>,
    /// Version of the block implementation.
    pub version: String,
}

/// Interface implemented by backend plugins.
///
/// Plugins can extend the system with new block kinds or other
/// functionality.  Each plugin must provide a unique name and may return
/// descriptors of additional blocks that should be available on the
/// frontend.
pub trait Plugin: Send + Sync {
    /// Unique plugin name.
    fn name(&self) -> &'static str;

    /// Version of the plugin API this implementation targets.
    fn version(&self) -> &str;

    /// Return block descriptors contributed by this plugin.
    fn blocks(&self) -> Vec<BlockDescriptor>;
}

/// Wrapper around a WebAssembly plugin.
///
/// The wasm module is expected to export four items:
/// `memory`, `name`, `version` and `blocks`.  Each function returns a
/// pointer and length pair of a UTF‑8 string inside the module's memory.
/// The string produced by `blocks` must be a JSON array convertible to
/// [`BlockDescriptor`].
pub struct WasmPlugin {
    name: &'static str,
    version: &'static str,
    blocks: Vec<BlockDescriptor>,
}

impl WasmPlugin {
    /// Load a wasm module and convert its output into a [`WasmPlugin`].
    ///
    /// The module must not import any functions and may only export the
    /// functions `name`, `version`, `blocks` and the default `memory`.
    /// Memory is limited to a single 64 KiB page.
    pub fn from_file(path: &Path) -> Option<Self> {
        let engine = Engine::default();
        let module = Module::from_file(&engine, path).ok()?;

        // Disallow any host imports.
        if module.imports().next().is_some() {
            return None;
        }

        // Ensure only expected exports are present.
        for export in module.exports() {
            match export.name() {
                "memory" | "name" | "version" | "blocks" => {}
                _ => return None,
            }
        }

        let mut store = Store::new(&engine, ());
        let instance = Instance::new(&mut store, &module, &[]).ok()?;
        let memory = instance.get_memory(&mut store, "memory")?;

        // Restrict memory to a single page (64 KiB).
        let limits = memory.ty(&store).limits();
        if limits.minimum() > 1 || limits.maximum().map_or(true, |m| m > 1) {
            return None;
        }

        fn read_string(
            store: &mut Store<()>,
            instance: &Instance,
            memory: &Memory,
            func: &str,
        ) -> Option<String> {
            let f = instance
                .get_typed_func::<(), (i32, i32)>(store, func)
                .ok()?;
            let (ptr, len) = f.call(store, ()).ok()?;
            let data = memory
                .data(store)
                .get(ptr as usize..(ptr as usize + len as usize))?;
            String::from_utf8(data.to_vec()).ok()
        }

        let name = read_string(&mut store, &instance, &memory, "name")?;
        let version = read_string(&mut store, &instance, &memory, "version")?;
        let blocks_json = read_string(&mut store, &instance, &memory, "blocks")?;
        let blocks: Vec<BlockDescriptor> = serde_json::from_str(&blocks_json).ok()?;

        Some(Self {
            name: Box::leak(name.into_boxed_str()),
            version: Box::leak(version.into_boxed_str()),
            blocks,
        })
    }
}

impl Plugin for WasmPlugin {
    fn name(&self) -> &'static str {
        self.name
    }

    fn version(&self) -> &str {
        self.version
    }

    fn blocks(&self) -> Vec<BlockDescriptor> {
        self.blocks.clone()
    }
}
