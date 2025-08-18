use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use tracing::error;

use super::VisualMeta;

#[derive(Default)]
struct Registry {
    metas: HashMap<String, VisualMeta>,
    dups: HashSet<String>,
}

static REGISTRY: Lazy<Mutex<Registry>> = Lazy::new(|| Mutex::new(Registry::default()));

/// Register a [`VisualMeta`] entry, tracking duplicates.
pub fn register(meta: VisualMeta) {
    match REGISTRY.lock() {
        Ok(mut reg) => {
            if reg.metas.contains_key(&meta.id) {
                reg.dups.insert(meta.id.clone());
            }
            reg.metas.insert(meta.id.clone(), meta);
        }
        Err(e) => error!("failed to lock ID registry for register: {e}"),
    }
}

/// Retrieve a [`VisualMeta`] by id.
pub fn get(id: &str) -> Option<VisualMeta> {
    match REGISTRY.lock() {
        Ok(reg) => reg.metas.get(id).cloned(),
        Err(e) => {
            error!("failed to lock ID registry for get: {e}");
            None
        }
    }
}

/// Return a list of duplicate ids encountered so far.
pub fn duplicates() -> Vec<String> {
    match REGISTRY.lock() {
        Ok(reg) => reg.dups.iter().cloned().collect(),
        Err(e) => {
            error!("failed to lock ID registry for duplicates: {e}");
            Vec::new()
        }
    }
}

/// Clear the registry. Useful for tests.
pub fn clear() {
    match REGISTRY.lock() {
        Ok(mut reg) => {
            reg.metas.clear();
            reg.dups.clear();
        }
        Err(e) => error!("failed to lock ID registry for clear: {e}"),
    }
}
