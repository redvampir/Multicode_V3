//! Translations used in settings-related UI text.

use super::Language;

#[derive(Debug, Clone, Copy)]
pub enum SettingsText {
    ConflictResolutionLabel,
    ConflictModePreferText,
    ConflictModePreferVisual,
}

pub fn settings_text(key: SettingsText, lang: Language) -> &'static str {
    use Language::*;
    match key {
        SettingsText::ConflictResolutionLabel => match lang {
            English => "Conflict resolution",
            Russian => "Решение конфликтов",
            Spanish => "Resolución de conflictos",
            German => "Konfliktlösung",
        },
        SettingsText::ConflictModePreferText => match lang {
            English => "Prefer Text",
            Russian => "Текст",
            Spanish => "Preferir texto",
            German => "Text bevorzugen",
        },
        SettingsText::ConflictModePreferVisual => match lang {
            English => "Prefer Visual",
            Russian => "Визуально",
            Spanish => "Preferir visual",
            German => "Visuell bevorzugen",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{settings_text, SettingsText};
    use crate::app::Language;

    #[test]
    fn settings_text_is_translated() {
        assert_eq!(
            settings_text(SettingsText::ConflictModePreferText, Language::English),
            "Prefer Text"
        );
        assert_eq!(
            settings_text(SettingsText::ConflictModePreferVisual, Language::Russian),
            "Визуально"
        );
    }
}
