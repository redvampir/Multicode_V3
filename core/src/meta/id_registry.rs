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

/// Регистрирует запись [`VisualMeta`], отслеживая дубликаты.
pub fn register(meta: VisualMeta) {
    match REGISTRY.lock() {
        Ok(mut reg) => {
            if reg.metas.contains_key(&meta.id) {
                reg.dups.insert(meta.id.clone());
            }
            reg.metas.insert(meta.id.clone(), meta);
        }
        Err(e) => error!("не удалось заблокировать реестр ID для регистрации: {e}"),
    }
}

/// Получает [`VisualMeta`] по идентификатору.
pub fn get(id: &str) -> Option<VisualMeta> {
    match REGISTRY.lock() {
        Ok(reg) => reg.metas.get(id).cloned(),
        Err(e) => {
            error!("не удалось заблокировать реестр ID для получения: {e}");
            None
        }
    }
}

/// Возвращает список встреченных дубликатов идентификаторов.
pub fn duplicates() -> Vec<String> {
    match REGISTRY.lock() {
        Ok(reg) => reg.dups.iter().cloned().collect(),
        Err(e) => {
            error!("не удалось заблокировать реестр ID при проверке дубликатов: {e}");
            Vec::new()
        }
    }
}

/// Очищает реестр. Полезно для тестов.
pub fn clear() {
    match REGISTRY.lock() {
        Ok(mut reg) => {
            reg.metas.clear();
            reg.dups.clear();
        }
        Err(e) => error!("не удалось заблокировать реестр ID для очистки: {e}"),
    }
}
