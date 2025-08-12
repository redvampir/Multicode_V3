use std::cmp::Ordering;
use std::collections::HashMap;

use chrono::Utc;
use syn::{File, Item};

use crate::{
    meta::{read_all, remove_all, upsert, VisualMeta},
    parser::{parse, parse_to_blocks, Lang},
    BlockInfo,
};

mod cache;
mod enrich;
mod parsing;

#[cfg_attr(not(test), tauri::command)]
pub fn parse_blocks(content: String, lang: String) -> Option<Vec<BlockInfo>> {
    let lang = match to_lang(&lang) {
        Some(l) => l,
        None => {
            tracing::error!("unsupported language: {}", lang);
            return None;
        }
    };

    let key = cache::key(&content);
    if let Some(blocks) = cache::get(&key, &content) {
        return Some(blocks);
    }

    let blocks = parsing::parse(&content, lang)?;
    let result = enrich::enrich_blocks(blocks, &content);
    cache::store(key, content, result.clone());
    Some(result)
}

#[cfg_attr(not(test), tauri::command)]
pub fn upsert_meta(content: String, mut meta: VisualMeta, lang: String) -> String {
    meta.updated_at = Utc::now();
    let mut metas = read_all(&content);
    if let Some(existing) = metas.iter().find(|m| m.id == meta.id) {
        if meta.version == 0 {
            meta.version = existing.version;
        }
        if meta.translations.is_empty() {
            meta.translations = existing.translations.clone();
        }
        if meta.ai.is_none() {
            meta.ai = existing.ai.clone();
        }
        if meta.tags.is_empty() {
            meta.tags = existing.tags.clone();
        }
        if meta.links.is_empty() {
            meta.links = existing.links.clone();
        }
    }
    if meta.version == 0 {
        meta.version = 1;
    }
    metas.retain(|m| m.id != meta.id);
    metas.push(meta);

    let cleaned = remove_all(&content);
    let lang = match to_lang(&lang) {
        Some(l) => l,
        None => {
            tracing::error!("unsupported language: {}", lang);
            return metas.into_iter().fold(cleaned, |acc, m| upsert(&acc, &m));
        }
    };
    let regenerated = regenerate_code(&cleaned, lang, &metas).unwrap_or(cleaned);

    metas
        .into_iter()
        .fold(regenerated, |acc, m| upsert(&acc, &m))
}

fn regenerate_code(content: &str, lang: Lang, metas: &[VisualMeta]) -> Option<String> {
    match lang {
        Lang::Rust => regenerate_rust(content, metas),
        _ => Some(content.to_string()),
    }
}

fn regenerate_rust(content: &str, metas: &[VisualMeta]) -> Option<String> {
    let mut file: File = syn::parse_file(content).ok()?;
    let tree = parse(content, Lang::Rust, None)?;
    let blocks = parse_to_blocks(&tree);
    let map: HashMap<_, _> = blocks
        .into_iter()
        .map(|b| (b.node_id, b.visual_id))
        .collect();

    let mut cursor = tree.root_node().walk();
    let mut roots = Vec::new();
    if cursor.goto_first_child() {
        loop {
            roots.push(cursor.node().id());
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }

    let mut items: Vec<(Item, (f64, f64))> = file
        .items
        .into_iter()
        .zip(roots.into_iter())
        .map(|(it, id)| {
            let vid = map.get(&(id as u32)).cloned().unwrap_or_default();
            let pos = metas
                .iter()
                .find(|m| m.id == vid)
                .map(|m| (m.y, m.x))
                .unwrap_or((0.0, 0.0));
            (it, pos)
        })
        .collect();

    items.sort_by(|a, b| {
        a.1 .0
            .partial_cmp(&b.1 .0)
            .unwrap_or(Ordering::Equal)
            .then_with(|| a.1 .1.partial_cmp(&b.1 .1).unwrap_or(Ordering::Equal))
    });

    file.items = items.into_iter().map(|(it, _)| it).collect();
    Some(prettyplease::unparse(&file))
}

pub fn to_lang(s: &str) -> Option<Lang> {
    match s.to_lowercase().as_str() {
        "rust" => Some(Lang::Rust),
        "python" => Some(Lang::Python),
        "javascript" => Some(Lang::JavaScript),
        "css" => Some(Lang::Css),
        "html" => Some(Lang::Html),
        _ => None,
    }
}
