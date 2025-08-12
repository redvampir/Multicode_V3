use std::cmp::Ordering;
use std::collections::{hash_map::DefaultHasher, HashMap};
use std::hash::{Hash, Hasher};

use chrono::Utc;
use syn::{File, Item};
use tree_sitter::{InputEdit, Point};

use crate::{
    get_cached_blocks, get_document_tree, i18n,
    meta::{read_all, remove_all, upsert, VisualMeta},
    parser::{parse, parse_to_blocks, Lang},
    update_block_cache, update_document_tree, BlockInfo,
};

#[cfg_attr(not(test), tauri::command)]
pub fn parse_blocks(content: String, lang: String) -> Option<Vec<BlockInfo>> {
    let lang = match to_lang(&lang) {
        Some(l) => l,
        None => {
            tracing::error!("unsupported language: {}", lang);
            return None;
        }
    };
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    let key = hasher.finish().to_string();
    if let Some(blocks) = get_cached_blocks(&key, &content) {
        return Some(blocks);
    }
    let old = get_document_tree("current");
    let tree = if let Some(mut old_tree) = old {
        let old_root = old_tree.root_node();
        let old_end_byte = old_root.end_byte();
        let old_end_position = old_root.end_position();
        let new_end_byte = content.as_bytes().len();
        let mut row = 0;
        let mut column = 0;
        for b in content.bytes() {
            if b == b'\n' {
                row += 1;
                column = 0;
            } else {
                column += 1;
            }
        }
        let new_end_position = Point { row, column };
        let edit = InputEdit {
            start_byte: 0,
            old_end_byte,
            new_end_byte,
            start_position: Point { row: 0, column: 0 },
            old_end_position,
            new_end_position,
        };
        old_tree.edit(&edit);
        parse(&content, lang, Some(&old_tree))?
    } else {
        parse(&content, lang, None)?
    };
    update_document_tree("current".to_string(), tree.clone());
    let blocks = parse_to_blocks(&tree);
    let metas = read_all(&content);
    let map: HashMap<_, _> = metas.into_iter().map(|m| (m.id.clone(), m)).collect();
    let result: Vec<BlockInfo> = blocks
        .into_iter()
        .map(|b| {
            let label = normalize_kind(&b.kind);
            let mut translations = i18n::lookup(&label).unwrap_or_else(|| {
                let mut m = HashMap::new();
                m.insert("ru".into(), label.clone());
                m.insert("en".into(), label.clone());
                m.insert("es".into(), label.clone());
                m
            });
            if let Some(meta) = map.get(&b.visual_id) {
                translations.extend(meta.translations.clone());
            }
            let pos = map.get(&b.visual_id);
            BlockInfo {
                visual_id: b.visual_id,
                node_id: Some(b.node_id),
                kind: label,
                translations,
                range: (b.range.start, b.range.end),
                x: pos.map(|m| m.x).unwrap_or(0.0),
                y: pos.map(|m| m.y).unwrap_or(0.0),
                ai: pos.and_then(|m| m.ai.clone()),
                links: pos.map(|m| m.links.clone()).unwrap_or_default(),
            }
        })
        .collect();
    update_block_cache(key, content, result.clone());
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

fn normalize_kind(kind: &str) -> String {
    let k = kind.to_lowercase();
    if k.contains("function") {
        "Function".into()
    } else if k.contains("if") {
        "Condition".into()
    } else if k.contains("for") || k.contains("while") || k.contains("loop") {
        "Loop".into()
    } else if k.contains("identifier") || k.contains("variable") {
        "Variable".into()
    } else {
        kind.to_string()
    }
}
