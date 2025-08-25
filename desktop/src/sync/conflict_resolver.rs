use multicode_core::meta::{AiNote, VisualMeta};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;

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

        resolved.version = std::cmp::max(text.version, visual.version);
        let conflict = SyncConflict {
            id: text.id.clone(),
            conflict_type,
            resolution,
        };
        (resolved, conflict)
    }
}

fn merge_strings(a: &[String], b: &[String]) -> Vec<String> {
    let mut set: HashSet<String> = a.iter().cloned().collect();
    for item in b {
        set.insert(item.clone());
    }
    let mut vec: Vec<String> = set.into_iter().collect();
    vec.sort();
    vec
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
        assert!(resolved.tags.contains(&"a".to_string()));
        assert!(resolved.tags.contains(&"b".to_string()));
        assert_eq!(conflict.conflict_type, ConflictType::MetaComment);
        assert_eq!(conflict.resolution, ResolutionOption::Merge);
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
}
