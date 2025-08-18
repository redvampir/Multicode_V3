//! Core library exposing language parsing, metadata handling and other utilities.

pub mod blocks;
pub mod config;
pub mod debugger;
#[cfg(feature = "export")]
pub mod export;
#[cfg(feature = "git")]
pub mod git;
pub mod i18n;
pub mod meta;
pub mod parser;
pub mod search;
pub mod viz_lint;

pub use blocks::{parse_blocks, upsert_meta};

use crate::meta::AiNote;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tree_sitter::Tree;

/// Parsed block information enriched with visual metadata.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct BlockInfo {
    pub visual_id: String,
    #[serde(default)]
    pub node_id: Option<u32>,
    pub kind: String,
    pub translations: HashMap<String, String>,
    pub range: (usize, usize),
    #[serde(default)]
    pub anchors: Vec<(usize, usize)>,
    pub x: f64,
    pub y: f64,
    pub ai: Option<AiNote>,
    #[serde(default)]
    pub links: Vec<String>,
}

/// Stored parse trees for opened documents.
static DOCUMENT_TREES: Lazy<Mutex<HashMap<String, Tree>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Cached parsed blocks keyed by path or content hash.
static BLOCK_CACHE: Lazy<Mutex<HashMap<String, (String, Vec<BlockInfo>)>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Retrieve the last parsed [`Tree`] for the given document identifier.
pub fn get_document_tree(id: &str) -> Option<Tree> {
    match DOCUMENT_TREES.lock() {
        Ok(trees) => trees.get(id).cloned(),
        Err(_) => None,
    }
}

/// Update the stored [`Tree`] for the given document identifier.
pub fn update_document_tree(id: String, tree: Tree) {
    if let Ok(mut trees) = DOCUMENT_TREES.lock() {
        trees.insert(id, tree);
    }
}

/// Retrieve cached blocks if the content matches.
pub fn get_cached_blocks(key: &str, content: &str) -> Option<Vec<BlockInfo>> {
    if let Ok(cache) = BLOCK_CACHE.lock() {
        if let Some((cached_content, blocks)) = cache.get(key) {
            if cached_content == content {
                return Some(blocks.clone());
            }
        }
    }
    None
}

/// Update the block cache for the given key.
pub fn update_block_cache(key: String, content: String, blocks: Vec<BlockInfo>) {
    if let Ok(mut cache) = BLOCK_CACHE.lock() {
        cache.insert(key, (content, blocks));
    }
}

