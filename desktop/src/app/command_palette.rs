#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandCategory {
    File,
    View,
    Navigation,
    Settings,
}

#[derive(Debug, Clone)]
pub struct CommandItem {
    pub id: &'static str,
    pub name_en: &'static str,
    pub name_ru: &'static str,
    pub description_en: &'static str,
    pub description_ru: &'static str,
    pub category: CommandCategory,
    pub hotkey: &'static str,
}

pub const COMMANDS: &[CommandItem] = &[
    CommandItem {
        id: "open_file",
        name_en: "Open File",
        name_ru: "Открыть файл",
        description_en: "Open a file from disk",
        description_ru: "Открыть файл с диска",
        category: CommandCategory::File,
        hotkey: "Ctrl+O",
    },
    CommandItem {
        id: "save_file",
        name_en: "Save File",
        name_ru: "Сохранить файл",
        description_en: "Save the current file",
        description_ru: "Сохранить текущий файл",
        category: CommandCategory::File,
        hotkey: "Ctrl+S",
    },
    CommandItem {
        id: "toggle_terminal",
        name_en: "Toggle Terminal",
        name_ru: "Показать/Скрыть терминал",
        description_en: "Show or hide the terminal",
        description_ru: "Показать или скрыть терминал",
        category: CommandCategory::View,
        hotkey: "Ctrl+`",
    },
    CommandItem {
        id: "goto_line",
        name_en: "Go to Line",
        name_ru: "Перейти к строке",
        description_en: "Jump to specified line number",
        description_ru: "Перейти к указанной строке",
        category: CommandCategory::Navigation,
        hotkey: "Ctrl+G",
    },
    CommandItem {
        id: "open_settings",
        name_en: "Open Settings",
        name_ru: "Открыть настройки",
        description_en: "Open application settings",
        description_ru: "Открыть настройки приложения",
        category: CommandCategory::Settings,
        hotkey: "Ctrl+,",
    },
    CommandItem {
        id: "switch_to_text_editor",
        name_en: "Switch to Text",
        name_ru: "Переключиться в текстовый редактор",
        description_en: "Switch to text editor",
        description_ru: "Переключиться в текстовый редактор",
        category: CommandCategory::View,
        hotkey: "Ctrl+1",
    },
    CommandItem {
        id: "switch_to_visual_editor",
        name_en: "Switch to Visual",
        name_ru: "Переключиться в визуальный редактор",
        description_en: "Switch to visual editor",
        description_ru: "Переключиться в визуальный редактор",
        category: CommandCategory::View,
        hotkey: "Ctrl+2",
    },
    CommandItem {
        id: "switch_to_split",
        name_en: "Switch to Split",
        name_ru: "Переключиться в режим разделения",
        description_en: "Switch to split view",
        description_ru: "Переключиться в режим разделения",
        category: CommandCategory::View,
        hotkey: "Ctrl+3",
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::command_translations::{command_hotkey, command_name};
    use crate::app::Language;

    #[test]
    fn commands_have_localizations() {
        let cmd = COMMANDS
            .iter()
            .find(|c| c.id == "open_file")
            .expect("open_file command not found");
        assert_eq!(command_name(cmd, Language::English), "Open File");
        assert_eq!(command_name(cmd, Language::Russian), "Открыть файл");
    }

    #[test]
    fn filtering_by_category_returns_view_commands() {
        let filtered: Vec<&CommandItem> = COMMANDS
            .iter()
            .filter(|c| c.category == CommandCategory::View)
            .collect();
        assert!(filtered.iter().any(|c| c.id == "switch_to_text_editor"));
        assert!(filtered.iter().any(|c| c.id == "switch_to_visual_editor"));
        assert!(filtered.iter().any(|c| c.id == "switch_to_split"));
    }

    #[test]
    fn all_commands_have_hotkeys() {
        assert!(COMMANDS.iter().all(|c| !c.hotkey.is_empty()));
    }

    #[test]
    fn hotkeys_are_translated() {
        let cmd = COMMANDS.iter().find(|c| c.id == "toggle_terminal").unwrap();
        #[cfg(target_os = "macos")]
        let prefix = "Cmd+";
        #[cfg(not(target_os = "macos"))]
        let prefix = "Ctrl+";
        assert_eq!(
            command_hotkey(cmd, Language::English),
            format!("{}{}", prefix, "`")
        );
        assert_eq!(
            command_hotkey(cmd, Language::Russian),
            format!("{}Ё", prefix)
        );
    }
}
