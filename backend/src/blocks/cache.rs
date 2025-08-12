use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::{get_cached_blocks, update_block_cache, BlockInfo};

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
