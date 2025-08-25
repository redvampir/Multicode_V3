use std::collections::HashSet;
use serde::{Deserialize, Serialize};

/// Delta produced by the text editor.
///
/// Contains identifiers of `@VISUAL_META` comments affected by an edit.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TextDelta {
    /// IDs of metadata entries touched by the text change.
    pub meta_ids: Vec<String>,
}

/// Delta produced by the visual editor.
///
/// Each delta is associated with identifiers of blocks described by
/// `VisualMeta` records.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VisualDelta {
    /// IDs of blocks that were changed in the visual editor.
    pub meta_ids: Vec<String>,
}

/// Tracks changes originating from text and visual editors.
#[derive(Debug, Default)]
pub struct ChangeTracker {
    text_changes: HashSet<String>,
    visual_changes: HashSet<String>,
}

impl ChangeTracker {
    /// Creates a new empty tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a delta produced by the text editor.
    pub fn record_text(&mut self, delta: TextDelta) {
        for id in delta.meta_ids {
            self.text_changes.insert(id);
        }
    }

    /// Registers a delta produced by the visual editor.
    pub fn record_visual(&mut self, delta: VisualDelta) {
        for id in delta.meta_ids {
            self.visual_changes.insert(id);
        }
    }

    /// Returns and clears accumulated text changes.
    ///
    /// The returned identifiers are sorted to ensure deterministic order.
    pub fn take_text_changes(&mut self) -> Vec<String> {
        let mut changes: Vec<_> = self.text_changes.drain().collect();
        changes.sort();
        changes
    }

    /// Returns and clears accumulated visual changes.
    ///
    /// The returned identifiers are sorted to ensure deterministic order.
    pub fn take_visual_changes(&mut self) -> Vec<String> {
        let mut changes: Vec<_> = self.visual_changes.drain().collect();
        changes.sort();
        changes
    }
}

#[cfg(test)]
mod tests {
    use super::{TextDelta, VisualDelta};
    use serde_json;

    #[test]
    fn text_delta_roundtrip() {
        let delta = TextDelta {
            meta_ids: vec!["a".to_string(), "b".to_string()],
        };
        let json = serde_json::to_string(&delta).unwrap();
        let deserialized: TextDelta = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.meta_ids, vec!["a", "b"]);
    }

    #[test]
    fn visual_delta_roundtrip() {
        let delta = VisualDelta {
            meta_ids: vec!["x".to_string()],
        };
        let json = serde_json::to_string(&delta).unwrap();
        let deserialized: VisualDelta = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.meta_ids, vec!["x"]);
    }
}
