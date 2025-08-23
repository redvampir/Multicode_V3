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
    pub category: CommandCategory,
    pub hotkey: &'static str,
}

pub const COMMANDS: &[CommandItem] = &[
    CommandItem {
        id: "open_file",
        category: CommandCategory::File,
        hotkey: "Ctrl+O",
    },
    CommandItem {
        id: "save_file",
        category: CommandCategory::File,
        hotkey: "Ctrl+S",
    },
    CommandItem {
        id: "toggle_terminal",
        category: CommandCategory::View,
        hotkey: "Ctrl+`",
    },
    CommandItem {
        id: "goto_line",
        category: CommandCategory::Navigation,
        hotkey: "Ctrl+G",
    },
    CommandItem {
        id: "open_settings",
        category: CommandCategory::Settings,
        hotkey: "Ctrl+,",
    },
    CommandItem {
        id: "switch_to_text_editor",
        category: CommandCategory::View,
        hotkey: "Ctrl+1",
    },
    CommandItem {
        id: "switch_to_visual_editor",
        category: CommandCategory::View,
        hotkey: "Ctrl+2",
    },
    CommandItem {
        id: "switch_to_split",
        category: CommandCategory::View,
        hotkey: "Ctrl+3",
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::command_translations::{command_hotkey, command_name};
    use crate::app::Language;
    use std::collections::HashSet;

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
    fn command_ids_are_unique() {
        let mut ids = HashSet::new();
        for cmd in COMMANDS {
            if !ids.insert(cmd.id) {
                panic!("duplicate command id: {}", cmd.id);
            }
        }
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
