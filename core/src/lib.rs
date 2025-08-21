//! Ядро библиотеки, предоставляющее парсинг языков, работу с метаданными и другие утилиты.

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

/// Информация о блоке, дополненная визуальными метаданными.
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
    /// Optional tags associated with the block.
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub links: Vec<String>,
}

/// Сохранённые деревья разбора для открытых документов.
static DOCUMENT_TREES: Lazy<Mutex<HashMap<String, Tree>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Кэш разобранных блоков, индексируемый путём или хешем содержимого.
static BLOCK_CACHE: Lazy<Mutex<HashMap<String, (String, Vec<BlockInfo>)>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Возвращает последнее разобранное [`Tree`] для указанного идентификатора документа.
pub fn get_document_tree(id: &str) -> Option<Tree> {
    match DOCUMENT_TREES.lock() {
        Ok(trees) => trees.get(id).cloned(),
        Err(_) => None,
    }
}

/// Обновляет сохранённое [`Tree`] для указанного идентификатора документа.
pub fn update_document_tree(id: String, tree: Tree) {
    if let Ok(mut trees) = DOCUMENT_TREES.lock() {
        trees.insert(id, tree);
    }
}

/// Возвращает кэшированные блоки, если содержимое совпадает.
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

/// Обновляет кэш блоков для заданного ключа.
pub fn update_block_cache(key: String, content: String, blocks: Vec<BlockInfo>) {
    if let Ok(mut cache) = BLOCK_CACHE.lock() {
        cache.insert(key, (content, blocks));
    }
}

