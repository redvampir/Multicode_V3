use std::path::Path;
use wasmtime::{Engine, Instance, Memory, Module, Store, StoreLimitsBuilder};

use super::{BlockDescriptor, Plugin};

/// Wrapper around a WebAssembly plugin.
///
/// The module is expected to export three functions returning
/// zero-terminated UTF-8 strings: `plugin_name`, `plugin_version` and
/// `plugin_blocks`.  The last one must produce a JSON array of
/// [`BlockDescriptor`] objects.  Only these functions and the default
/// memory export are permitted.
pub struct WasmPlugin {
    name: &'static str,
    version: String,
    blocks: Vec<BlockDescriptor>,
}

impl Plugin for WasmPlugin {
    fn name(&self) -> &'static str {
        self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn blocks(&self) -> Vec<BlockDescriptor> {
        self.blocks.clone()
    }
}

impl WasmPlugin {
    /// Load a plugin from the given WebAssembly module.
    pub fn from_file(path: &Path) -> Option<Self> {
        let engine = Engine::default();
        let module = Module::from_file(&engine, path).ok()?;

        // Ensure only the expected functions and memory are exported.
        let allowed = ["plugin_name", "plugin_version", "plugin_blocks", "memory"];
        for export in module.exports() {
            if !allowed.contains(&export.name()) {
                return None;
            }
        }

        let mut store = Store::new(&engine, ());
        let mut limits = StoreLimitsBuilder::new().memory_size(1 << 20).instances(1).build();
        store.limiter(|_| &mut limits);

        let instance = Instance::new(&mut store, &module, &[]).ok()?;
        let memory = instance.get_memory(&mut store, "memory")?;

        let name_ptr: i32 = instance
            .get_typed_func::<(), i32>(&mut store, "plugin_name")
            .ok()?
            .call(&mut store, ())
            .ok()?;
        let version_ptr: i32 = instance
            .get_typed_func::<(), i32>(&mut store, "plugin_version")
            .ok()?
            .call(&mut store, ())
            .ok()?;
        let blocks_ptr: i32 = instance
            .get_typed_func::<(), i32>(&mut store, "plugin_blocks")
            .ok()?
            .call(&mut store, ())
            .ok()?;

        let name = read_cstr(&mut store, &memory, name_ptr)?;
        let version = read_cstr(&mut store, &memory, version_ptr)?;
        let blocks_json = read_cstr(&mut store, &memory, blocks_ptr)?;
        let blocks: Vec<BlockDescriptor> = serde_json::from_str(&blocks_json).ok()?;

        let name: &'static str = Box::leak(name.into_boxed_str());
        Some(Self { name, version, blocks })
    }
}

fn read_cstr(store: &mut Store<()>, memory: &Memory, ptr: i32) -> Option<String> {
    let data = memory.data(store);
    let mut end = ptr as usize;
    while end < data.len() && data[end] != 0 {
        end += 1;
    }
    if end >= data.len() {
        return None;
    }
    let bytes = &data[ptr as usize..end];
    String::from_utf8(bytes.to_vec()).ok()
}
