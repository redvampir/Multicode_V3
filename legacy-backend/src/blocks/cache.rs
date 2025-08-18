use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::{get_cached_blocks, parser::Block, update_block_cache, BlockInfo};

/// Generate a stable cache key based on the file `content`.
pub fn key(content: &str) -> String {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish().to_string()
}

/// Try to retrieve cached block information for `key` and `content`.
pub fn get(key: &str, content: &str) -> Option<Vec<BlockInfo>> {
    get_cached_blocks(key, content)
}

/// Store parsed block information in the cache under `key`.
pub fn store(key: String, content: String, blocks: Vec<BlockInfo>) {
    update_block_cache(key, content, blocks);
}

/// Generate a stable identifier for a block based on its `range` and `content`.
///
/// The produced id is deterministic for the same snippet occupying the same
/// position which allows us to preserve UI state (anchors, coordinates) between
/// parse runs.
pub fn stable_id(content: &str, range: (usize, usize)) -> String {
    let mut hasher = DefaultHasher::new();
    let start = range.0.min(content.len());
    let end = range.1.min(content.len());
    let snippet = content[start..end].trim();
    snippet.hash(&mut hasher);
    start.hash(&mut hasher);
    hasher.finish().to_string()
}

/// Assign stable identifiers to all parsed `blocks` using the current `content`.
pub fn assign_ids(content: &str, blocks: &mut [Block]) {
    for b in blocks.iter_mut() {
        b.visual_id = stable_id(content, (b.range.start, b.range.end));
    }
}
