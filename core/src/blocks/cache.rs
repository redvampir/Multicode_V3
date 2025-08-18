use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::{get_cached_blocks, parser::Block, update_block_cache, BlockInfo};

/// Генерирует стабильный ключ кэша на основе содержимого файла `content`.
pub fn key(content: &str) -> String {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish().to_string()
}

/// Пытается получить закэшированную информацию о блоках для `key` и `content`.
pub fn get(key: &str, content: &str) -> Option<Vec<BlockInfo>> {
    get_cached_blocks(key, content)
}

/// Сохраняет разобранные блоки в кэше под ключом `key`.
pub fn store(key: String, content: String, blocks: Vec<BlockInfo>) {
    update_block_cache(key, content, blocks);
}

/// Генерирует стабильный идентификатор для блока на основе его `range` и `content`.
///
/// Полученный идентификатор детерминирован для одного и того же фрагмента,
/// занимающего одно и то же положение, что позволяет сохранять состояние UI
/// (якоря, координаты) между запусками парсера.
pub fn stable_id(content: &str, range: (usize, usize)) -> String {
    let mut hasher = DefaultHasher::new();
    let start = range.0.min(content.len());
    let end = range.1.min(content.len());
    let snippet = content[start..end].trim();
    snippet.hash(&mut hasher);
    start.hash(&mut hasher);
    hasher.finish().to_string()
}

/// Присваивает стабильные идентификаторы всем разобранным `blocks`, используя текущее `content`.
pub fn assign_ids(content: &str, blocks: &mut [Block]) {
    for b in blocks.iter_mut() {
        b.visual_id = stable_id(content, (b.range.start, b.range.end));
    }
}
