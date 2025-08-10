use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use once_cell::sync::Lazy;

use super::VisualMeta;

#[derive(Default)]
struct Registry {
    metas: HashMap<String, VisualMeta>,
    dups: HashSet<String>,
}

static REGISTRY: Lazy<Mutex<Registry>> = Lazy::new(|| Mutex::new(Registry::default()));

/// Register a [`VisualMeta`] entry, tracking duplicates.
pub fn register(meta: VisualMeta) {
    let mut reg = REGISTRY.lock().unwrap();
    if reg.metas.contains_key(&meta.id) {
        reg.dups.insert(meta.id.clone());
    }
    reg.metas.insert(meta.id.clone(), meta);
}

/// Retrieve a [`VisualMeta`] by id.
pub fn get(id: &str) -> Option<VisualMeta> {
    REGISTRY.lock().unwrap().metas.get(id).cloned()
}

/// Return a list of duplicate ids encountered so far.
pub fn duplicates() -> Vec<String> {
    REGISTRY
        .lock()
        .unwrap()
        .dups
        .iter()
        .cloned()
        .collect()
}

/// Clear the registry. Useful for tests.
pub fn clear() {
    let mut reg = REGISTRY.lock().unwrap();
    reg.metas.clear();
    reg.dups.clear();
}
