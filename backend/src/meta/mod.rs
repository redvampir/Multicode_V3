use chrono::Utc;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
mod comment_detector;
pub mod db;
pub mod id_registry;
mod types;
pub use types::{AiNote, VisualMeta};

/// Marker used to identify visual metadata comments in documents.
const MARKER: &str = "@VISUAL_META";

fn migrate(meta: &mut VisualMeta) {
    if meta.version == 0 {
        meta.version = 1;
    }
}

/// Structured validation error for [`VisualMeta`].
#[derive(Debug, Serialize)]
pub struct ValidationError {
    /// Field where validation failed.
    pub field: String,
    /// Description of the validation issue.
    pub message: String,
}

/// Validate a [`VisualMeta`] instance.
pub fn validate(meta: &VisualMeta) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    if meta.id.trim().is_empty() {
        errors.push(ValidationError {
            field: "id".into(),
            message: "id must not be empty".into(),
        });
    }

    if !meta.x.is_finite() {
        errors.push(ValidationError {
            field: "x".into(),
            message: "x must be a finite number".into(),
        });
    }

    if !meta.y.is_finite() {
        errors.push(ValidationError {
            field: "y".into(),
            message: "y must be a finite number".into(),
        });
    }

    let mut tag_set = HashSet::new();
    for tag in &meta.tags {
        if !tag_set.insert(tag) {
            errors.push(ValidationError {
                field: "tags".into(),
                message: format!("duplicate tag '{tag}'"),
            });
        }
    }

    let mut link_set = HashSet::new();
    for link in &meta.links {
        if !link_set.insert(link) {
            errors.push(ValidationError {
                field: "links".into(),
                message: format!("duplicate link '{link}'"),
            });
        }
    }

    if let Some(ext) = &meta.extends {
        if ext.trim().is_empty() {
            errors.push(ValidationError {
                field: "extends".into(),
                message: "extends must not be empty".into(),
            });
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Insert or update a visual metadata comment in `content`.
///
/// The comment will be placed at the top of the document if it does not exist.
pub fn upsert(content: &str, meta: &VisualMeta) -> String {
    let marker = format!("<!-- {} ", MARKER);
    let mut meta = meta.clone();
    meta.updated_at = Utc::now();
    let serialized = match serde_json::to_string(&meta) {
        Ok(s) => s,
        Err(_) => return content.to_string(),
    };

    let mut out = String::new();
    let mut found = false;
    for line in content.lines() {
        if line.trim_start().starts_with(&marker) {
            if let Some(end_idx) = line.find("-->") {
                let json_part = &line[marker.len()..end_idx].trim();
                if let Ok(existing) = serde_json::from_str::<VisualMeta>(json_part) {
                    if existing.id == meta.id {
                        out.push_str(&format!("{}{} -->\n", marker, serialized));
                        found = true;
                        continue;
                    }
                }
            }
        }
        out.push_str(line);
        out.push('\n');
    }

    if !found {
        out = format!("{}{} -->\n{}", marker, serialized, out);
    }

    out
}

/// Read all visual metadata comments from `content`.
pub fn read_all(content: &str) -> Vec<VisualMeta> {
    id_registry::clear();
    let mut ids = Vec::new();
    for json in comment_detector::extract_json(content) {
        if let Ok(mut meta) = serde_json::from_str::<VisualMeta>(&json) {
            migrate(&mut meta);
            let id = meta.id.clone();
            id_registry::register(meta);
            ids.push(id);
        }
    }
    ids.into_iter()
        .filter_map(|id| merge_base_meta(&id))
        .collect()
}

/// Recursively merge metadata with its base entries using the `extends` chain.
pub fn merge_base_meta(id: &str) -> Option<VisualMeta> {
    fn inner(id: &str, visited: &mut HashSet<String>) -> Option<VisualMeta> {
        if !visited.insert(id.to_string()) {
            return id_registry::get(id);
        }
        let mut meta = id_registry::get(id)?;
        if let Some(parent_id) = meta.extends.clone() {
            if let Some(base) = inner(&parent_id, visited) {
                meta = merge_two(base, meta);
            }
        }
        meta.extends = None;
        Some(meta)
    }

    fn merge_two(mut base: VisualMeta, mut child: VisualMeta) -> VisualMeta {
        for tag in base.tags {
            if !child.tags.contains(&tag) {
                child.tags.insert(0, tag);
            }
        }
        for link in base.links {
            if !child.links.contains(&link) {
                child.links.insert(0, link);
            }
        }
        if child.origin.is_none() {
            child.origin = base.origin;
        }
        for (k, v) in base.translations {
            child.translations.entry(k).or_insert(v);
        }
        child.ai = match (child.ai, base.ai) {
            (Some(mut c), Some(b)) => {
                if c.description.is_none() {
                    c.description = b.description;
                }
                for hint in b.hints.into_iter().rev() {
                    if !c.hints.contains(&hint) {
                        c.hints.insert(0, hint);
                    }
                }
                Some(c)
            }
            (Some(c), None) => Some(c),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        };
        if child.extras.is_none() {
            child.extras = base.extras;
        }
        if child.updated_at.timestamp() == 0 {
            child.updated_at = base.updated_at;
        }
        child
    }

    inner(id, &mut HashSet::new())
}

/// Remove all visual metadata comments from `content`.
pub fn remove_all(content: &str) -> String {
    comment_detector::strip(content)
}

/// Convenience wrapper returning all visual metadata entries from `content`.
pub fn list(content: &str) -> Vec<VisualMeta> {
    read_all(content)
}

/// Fix issues in metadata comments, such as duplicate identifiers.
///
/// When duplicate ids are found, new unique ones are generated and the
/// updated metadata comments are reinserted into the document.
pub fn fix_all(content: &str) -> String {
    let mut metas = read_all(content);
    let mut seen = HashSet::new();
    let mut changed = false;
    for meta in &mut metas {
        if !seen.insert(meta.id.clone()) {
            meta.id = unique_id();
            changed = true;
        }
    }

    if !changed {
        return content.to_string();
    }

    let mut out = remove_all(content);
    for meta in &metas {
        out = upsert(&out, meta);
    }
    out
}

fn unique_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("m{}", nanos)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn upsert_and_read_roundtrip() {
        let meta = VisualMeta {
            version: 1,
            id: "1".into(),
            x: 10.0,
            y: 20.0,
            tags: vec!["alpha".into(), "beta".into()],
            links: vec![],
            extends: None,
            origin: None,
            translations: HashMap::new(),
            ai: Some(AiNote {
                description: Some("desc".into()),
                hints: vec!["hint".into()],
            }),
            extras: Some(json!({"foo": "bar"})),
            updated_at: Utc::now(),
        };
        let content = "fn main() {}";
        let updated = upsert(content, &meta);
        assert!(updated.contains(MARKER));
        let metas = read_all(&updated);
        assert_eq!(metas.len(), 1);
        assert_eq!(metas[0].x, 10.0);
        assert_eq!(metas[0].tags, vec!["alpha", "beta"]);
        assert!(metas[0].links.is_empty());
        assert_eq!(
            metas[0].ai.as_ref().unwrap().description.as_deref(),
            Some("desc")
        );
        assert_eq!(metas[0].extras, Some(json!({"foo": "bar"})));
        assert!(metas[0].updated_at.timestamp() > 0);
        assert_eq!(metas[0].version, 1);
    }

    #[test]
    fn remove_all_strips_metadata() {
        let content = format!("line1\n<!-- {} {{\"id\":\"1\"}} -->\nline2\n", MARKER);
        let cleaned = remove_all(&content);
        assert!(!cleaned.contains(MARKER));
        assert!(cleaned.contains("line1"));
        assert!(cleaned.contains("line2"));
    }

    #[test]
    fn fix_all_replaces_duplicate_ids() {
        let content = format!(
            "<!-- {} {{\"id\":\"1\",\"x\":0.0,\"y\":0.0}} -->\n<!-- {} {{\"id\":\"1\",\"x\":1.0,\"y\":1.0}} -->",
            MARKER, MARKER
        );
        let fixed = fix_all(&content);
        let metas = read_all(&fixed);
        assert_eq!(metas.len(), 2);
        assert_ne!(metas[0].id, metas[1].id);
    }

    #[test]
    fn merge_base_meta_combines_fields() {
        let parent = VisualMeta {
            version: 1,
            id: "p".into(),
            x: 0.0,
            y: 0.0,
            tags: vec!["base".into()],
            links: vec!["l1".into()],
            extends: None,
            origin: Some("orig".into()),
            translations: {
                let mut m = HashMap::new();
                m.insert("en".into(), "Parent".into());
                m
            },
            ai: Some(AiNote {
                description: Some("pdesc".into()),
                hints: vec!["h1".into()],
            }),
            extras: None,
            updated_at: Utc::now(),
        };

        let child = VisualMeta {
            version: 1,
            id: "c".into(),
            x: 1.0,
            y: 1.0,
            tags: vec!["child".into()],
            links: vec!["l2".into()],
            extends: Some("p".into()),
            origin: None,
            translations: {
                let mut m = HashMap::new();
                m.insert("ru".into(), "Дитя".into());
                m
            },
            ai: Some(AiNote {
                description: None,
                hints: vec!["h2".into()],
            }),
            extras: None,
            updated_at: Utc::now(),
        };

        let content = format!(
            "// @VISUAL_META {}\n// @VISUAL_META {}\n",
            serde_json::to_string(&parent).unwrap(),
            serde_json::to_string(&child).unwrap()
        );

        let metas = read_all(&content);
        let merged = metas.iter().find(|m| m.id == "c").unwrap();
        assert_eq!(merged.tags, vec!["base", "child"]);
        assert_eq!(merged.links, vec!["l1", "l2"]);
        assert_eq!(merged.origin.as_deref(), Some("orig"));
        assert_eq!(merged.translations.get("en").unwrap(), "Parent");
        assert_eq!(merged.translations.get("ru").unwrap(), "Дитя");
        let ai = merged.ai.as_ref().unwrap();
        assert_eq!(ai.description.as_deref(), Some("pdesc"));
        assert_eq!(ai.hints, vec!["h1", "h2"]);
    }

    #[test]
    fn validate_reports_errors() {
        let meta = VisualMeta {
            version: 1,
            id: "".into(),
            x: f64::NAN,
            y: 0.0,
            tags: vec!["a".into(), "a".into()],
            links: vec!["l".into(), "l".into()],
            extends: Some("".into()),
            origin: None,
            translations: HashMap::new(),
            ai: None,
            extras: None,
            updated_at: Utc::now(),
        };
        let errs = validate(&meta).unwrap_err();
        assert!(errs.iter().any(|e| e.field == "id"));
        assert!(errs.iter().any(|e| e.field == "x"));
        assert!(errs.iter().any(|e| e.field == "tags"));
        assert!(errs.iter().any(|e| e.field == "links"));
        assert!(errs.iter().any(|e| e.field == "extends"));
    }
}
