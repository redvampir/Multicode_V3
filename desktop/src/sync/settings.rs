use serde::{Deserialize, Serialize};

use super::conflict_resolver::ResolutionPolicy;
use crate::app::{settings_translations::{settings_text, SettingsText}, Language};

fn default_conflict_mode() -> ConflictResolutionMode {
    ConflictResolutionMode::PreferText
}

fn default_true() -> bool {
    true
}

/// Available strategies for resolving conflicts between text and visual metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictResolutionMode {
    /// Prefer textual representation when conflicts arise.
    PreferText,
    /// Prefer visual representation when conflicts arise.
    PreferVisual,
}

impl ConflictResolutionMode {
    /// All variants for use in user interfaces.
    pub const ALL: [ConflictResolutionMode; 2] = [
        ConflictResolutionMode::PreferText,
        ConflictResolutionMode::PreferVisual,
    ];

    pub fn label(self, lang: Language) -> &'static str {
        match self {
            ConflictResolutionMode::PreferText =>
                settings_text(SettingsText::ConflictModePreferText, lang),
            ConflictResolutionMode::PreferVisual =>
                settings_text(SettingsText::ConflictModePreferVisual, lang),
        }
    }
}

impl Default for ConflictResolutionMode {
    fn default() -> Self {
        default_conflict_mode()
    }
}

impl From<ConflictResolutionMode> for ResolutionPolicy {
    fn from(mode: ConflictResolutionMode) -> Self {
        match mode {
            ConflictResolutionMode::PreferText => ResolutionPolicy::PreferText,
            ConflictResolutionMode::PreferVisual => ResolutionPolicy::PreferVisual,
        }
    }
}

/// Synchronization settings affecting conflict resolution behaviour.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSettings {
    /// Strategy used to resolve conflicts between text and visual metadata.
    #[serde(default = "default_conflict_mode")]
    pub conflict_resolution: ConflictResolutionMode,
    /// Preserve formatting of existing meta comments when updating code.
    #[serde(default = "default_true")]
    pub preserve_meta_formatting: bool,
}

impl Default for SyncSettings {
    fn default() -> Self {
        Self {
            conflict_resolution: default_conflict_mode(),
            preserve_meta_formatting: default_true(),
        }
    }
}
