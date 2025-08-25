use multicode_core::meta::{AiNote, VisualMeta};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use indexmap::IndexSet;
use std::hash::Hash;

/// Type of conflict detected between text and visual representations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictType {
    /// Differences in structural metadata (e.g. translations, extends).
    Structural,
    /// Block was moved on canvas (coordinate differences).
    Movement,
    /// Metadata comments such as tags or links diverged.
    MetaComment,
}

/// Resolution option applied to a conflict.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolutionOption {
    /// Prefer text representation.
    Text,
    /// Prefer visual representation.
    Visual,
    /// Merge both representations.
    Merge,
}

/// Conflict description bound to a specific `VisualMeta` identifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConflict {
    /// Identifier of the metadata entry that produced the conflict.
    pub id: String,
    /// Kind of conflict.
    pub conflict_type: ConflictType,
    /// Chosen resolution strategy.
    pub resolution: ResolutionOption,
}

/// Resolves conflicts between two versions of [`VisualMeta`].
#[derive(Debug, Default)]
pub struct ConflictResolver;

impl ConflictResolver {
    /// Resolve conflict between text and visual versions of a metadata entry.
    pub fn resolve(&self, text: &VisualMeta, visual: &VisualMeta) -> (VisualMeta, SyncConflict) {
        use ConflictType::*;
        use ResolutionOption::*;

        let movement = text.x != visual.x || text.y != visual.y;
        let meta_diff = text.tags != visual.tags
            || text.links != visual.links
            || text.anchors != visual.anchors
            || text.tests != visual.tests
            || !ainote_eq(&text.ai, &visual.ai)
            || text.extras != visual.extras;
        let structural_diff = text.translations != visual.translations
            || text.extends != visual.extends
            || text.origin != visual.origin;

        let mut resolved = text.clone();
        if movement {
            resolved.x = visual.x;
            resolved.y = visual.y;
        }
        if meta_diff {
            resolved.tags = merge_strings(&text.tags, &visual.tags);
            resolved.links = merge_strings(&text.links, &visual.links);
            resolved.anchors = merge_strings(&text.anchors, &visual.anchors);
            resolved.tests = merge_strings(&text.tests, &visual.tests);
            resolved.ai = merge_ai(&text.ai, &visual.ai);
            resolved.extras = merge_json(&text.extras, &visual.extras);
        }

        let (conflict_type, resolution) = if structural_diff {
            (Structural, Text)
        } else if movement {
            (Movement, if meta_diff { Merge } else { Visual })
        } else {
            (MetaComment, Merge)
        };

        match conflict_type {
            Structural => tracing::warn!(id = %text.id, "Structural conflict detected"),
            _ => {
                tracing::debug!(id = %text.id, conflict_type = ?conflict_type, "Conflict detected")
            }
        }
        tracing::debug!(id = %text.id, resolution = ?resolution, "Conflict resolved");

        resolved.version = std::cmp::max(text.version, visual.version);
        let conflict = SyncConflict {
            id: text.id.clone(),
            conflict_type,
            resolution,
        };
        (resolved, conflict)
    }
}

fn merge_vec<T: Eq + Hash + Clone>(a: &[T], b: &[T]) -> Vec<T> {
    let mut set: IndexSet<T> = IndexSet::new();
    set.extend(a.iter().cloned());
    set.extend(b.iter().cloned());
    set.into_iter().collect()
}

fn merge_strings(a: &[String], b: &[String]) -> Vec<String> {
    merge_vec(a, b)
}

fn merge_json(a: &Option<Value>, b: &Option<Value>) -> Option<Value> {
    match (a, b) {
        (Some(Value::Object(map_a)), Some(Value::Object(map_b))) => {
            let mut merged = map_a.clone();
            for (k, v) in map_b.iter() {
                merged.insert(k.clone(), v.clone());
            }
            Some(Value::Object(merged))
        }
        (_, Some(v)) => Some(v.clone()),
        (Some(v), None) => Some(v.clone()),
        (None, None) => None,
    }
}

fn ainote_eq(a: &Option<AiNote>, b: &Option<AiNote>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => a.description == b.description && a.hints == b.hints,
        _ => false,
    }
}

fn merge_ai(a: &Option<AiNote>, b: &Option<AiNote>) -> Option<AiNote> {
    match (a, b) {
        (Some(a), Some(b)) => {
            let description = b.description.clone().or_else(|| a.description.clone());
            let hints = merge_strings(&a.hints, &b.hints);
            Some(AiNote { description, hints })
        }
        (_, Some(b)) => Some(b.clone()),
        (Some(a), None) => Some(a.clone()),
        (None, None) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::collections::HashMap;

    fn meta(id: &str) -> VisualMeta {
        VisualMeta {
            version: 1,
            id: id.into(),
            x: 0.0,
            y: 0.0,
            tags: vec![],
            links: vec![],
            anchors: vec![],
            tests: vec![],
            extends: None,
            origin: None,
            translations: HashMap::new(),
            ai: None,
            extras: None,
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn movement_prefers_visual() {
        let text = meta("1");
        let mut visual = meta("1");
        visual.version = 2;
        visual.x = 10.0;
        visual.y = 5.0;
        let (resolved, conflict) = ConflictResolver::default().resolve(&text, &visual);
        assert_eq!(resolved.x, 10.0);
        assert_eq!(resolved.y, 5.0);
        assert_eq!(conflict.conflict_type, ConflictType::Movement);
        assert_eq!(conflict.resolution, ResolutionOption::Visual);
    }

    #[test]
    fn meta_conflict_merges() {
        let mut text = meta("1");
        text.tags = vec!["a".into()];
        let mut visual = meta("1");
        visual.version = 2;
        visual.tags = vec!["b".into()];
        let (resolved, conflict) = ConflictResolver::default().resolve(&text, &visual);
        assert_eq!(resolved.tags, vec!["a".to_string(), "b".to_string()]);
        assert_eq!(conflict.conflict_type, ConflictType::MetaComment);
        assert_eq!(conflict.resolution, ResolutionOption::Merge);
    }

    #[test]
    fn merge_strings_preserves_order() {
        let text = vec!["a".to_string(), "c".to_string()];
        let visual = vec!["c".to_string(), "b".to_string()];
        let merged = super::merge_strings(&text, &visual);
        assert_eq!(merged, vec!["a".to_string(), "c".to_string(), "b".to_string()]);
    }

    #[test]
    fn structural_prefers_text() {
        let mut text = meta("1");
        text.translations
            .insert("rust".into(), "fn main() {}".into());
        let mut visual = meta("1");
        visual.version = 2;
        visual.translations.insert("rust".into(), "changed".into());
        let (resolved, conflict) = ConflictResolver::default().resolve(&text, &visual);
        assert_eq!(resolved.translations.get("rust").unwrap(), "fn main() {}");
        assert_eq!(conflict.conflict_type, ConflictType::Structural);
        assert_eq!(conflict.resolution, ResolutionOption::Text);
    }

    #[test]
    fn detect_structural_conflict() {
        let mut text = meta("1");
        text.extends = Some("base".into());
        let mut visual = meta("1");
        visual.version = 2;
        visual.extends = Some("changed".into());
        let (_, conflict) = ConflictResolver::default().resolve(&text, &visual);
        assert_eq!(conflict.conflict_type, ConflictType::Structural);
        assert_eq!(conflict.resolution, ResolutionOption::Text);
    }

    #[test]
    fn detect_meta_comment_conflict() {
        let mut text = meta("1");
        text.links = vec!["a".into()];
        let mut visual = meta("1");
        visual.version = 2;
        visual.links = vec!["b".into()];
        let (_, conflict) = ConflictResolver::default().resolve(&text, &visual);
        assert_eq!(conflict.conflict_type, ConflictType::MetaComment);
        assert_eq!(conflict.resolution, ResolutionOption::Merge);
    }
}
