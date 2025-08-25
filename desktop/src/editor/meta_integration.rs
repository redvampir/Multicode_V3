use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::ops::Range;

use multicode_core::meta::{self, VisualMeta};

use crate::app::Diagnostic;

static META_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"@VISUAL_META\s*(\{.*?\})").unwrap());

/// Find all `@VISUAL_META` comments in `content`.
/// Returns tuples of `(line_index, json_range, json)`.
///
/// The JSON portion is matched using a non-greedy pattern and therefore
/// supports multiple `@VISUAL_META` comments per line and ignores trailing
/// text after the closing brace. Nested braces or braces inside strings are
/// not handled and may cause incorrect matches.
pub fn find_meta_comments(content: &str) -> Vec<(usize, Range<usize>, String)> {
    let mut out = Vec::new();
    for (line_idx, line) in content.lines().enumerate() {
        for caps in META_REGEX.captures_iter(line) {
            if let Some(m) = caps.get(1) {
                out.push((line_idx, m.start()..m.end(), m.as_str().to_string()));
            }
        }
    }
    out
}

/// Determine which `@VISUAL_META` comments changed between two versions of
/// the content and return their identifiers.
pub fn changed_meta_ids(old: &str, new: &str) -> Vec<String> {
    let mut old_map = HashMap::new();
    for (_, _, json) in find_meta_comments(old) {
        if let Ok(meta) = serde_json::from_str::<VisualMeta>(&json) {
            old_map.insert(meta.id, json);
        }
    }
    let mut new_map = HashMap::new();
    for (_, _, json) in find_meta_comments(new) {
        if let Ok(meta) = serde_json::from_str::<VisualMeta>(&json) {
            new_map.insert(meta.id, json);
        }
    }
    let mut ids: HashSet<&String> = HashSet::new();
    ids.extend(old_map.keys());
    ids.extend(new_map.keys());

    let mut changed: HashSet<String> = HashSet::new();
    for id in ids {
        if old_map.get(id) != new_map.get(id) {
            changed.insert(id.clone());
        }
    }
    changed.into_iter().collect()
}

/// Insert a new visual meta comment into `content`.
pub fn insert_meta_comment(content: &str, meta: &VisualMeta) -> String {
    meta::upsert(content, meta)
}

/// Update existing `@VISUAL_META` comment or insert if missing.
pub fn update_meta_comment(content: &str, meta: &VisualMeta) -> String {
    meta::upsert(content, meta)
}

/// Validate JSON inside `@VISUAL_META` comments and produce diagnostics.
pub fn validate_meta_json(content: &str) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for (line, range, json) in find_meta_comments(content) {
        match serde_json::from_str::<VisualMeta>(&json) {
            Ok(meta) => {
                if let Err(errors) = meta::validate(&meta) {
                    for e in errors {
                        diags.push(Diagnostic {
                            line,
                            range: range.clone(),
                            message: format!("{}: {}", e.field, e.message),
                        });
                    }
                }
            }
            Err(e) => diags.push(Diagnostic {
                line,
                range,
                message: e.to_string(),
            }),
        }
    }
    diags
}
