use crate::app::events::Message;

#[derive(Debug, Clone)]
pub struct CommandItem {
    pub id: &'static str,
    pub title: &'static str,
    pub message: Message,
}

pub const COMMANDS: &[CommandItem] = &[
    CommandItem {
        id: "open_file",
        title: "Открыть файл",
        message: Message::PickFile,
    },
    CommandItem {
        id: "save_file",
        title: "Сохранить файл",
        message: Message::SaveFile,
    },
    CommandItem {
        id: "toggle_terminal",
        title: "Показать/Скрыть терминал",
        message: Message::ToggleTerminal,
    },
    CommandItem {
        id: "open_settings",
        title: "Открыть настройки",
        message: Message::OpenSettings,
    },
    CommandItem {
        id: "switch_to_text_editor",
        title: "Switch to Text",
        message: Message::SwitchToTextEditor,
    },
    CommandItem {
        id: "switch_to_visual_editor",
        title: "Switch to Visual",
        message: Message::SwitchToVisualEditor,
    },
    CommandItem {
        id: "switch_to_split",
        title: "Switch to Split",
        message: Message::SwitchViewMode(crate::app::ViewMode::Split),
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_palette_contains_switch_commands() {
        assert!(COMMANDS
            .iter()
            .any(|c| c.id == "switch_to_text_editor"
                && matches!(c.message, Message::SwitchToTextEditor)));
        assert!(COMMANDS.iter().any(|c| c.id == "switch_to_visual_editor"
            && matches!(c.message, Message::SwitchToVisualEditor)));
        assert!(COMMANDS.iter().any(|c| c.id == "switch_to_split"
            && matches!(c.message, Message::SwitchViewMode(crate::app::ViewMode::Split))));
    }

    #[test]
    fn filtering_by_query_returns_switch_commands() {
        let query = "switch";
        let filtered: Vec<&CommandItem> = COMMANDS
            .iter()
            .filter(|c| c.title.to_lowercase().contains(&query.to_lowercase()))
            .collect();
        assert!(filtered.iter().any(|c| c.id == "switch_to_text_editor"));
        assert!(filtered.iter().any(|c| c.id == "switch_to_visual_editor"));
        assert!(filtered.iter().any(|c| c.id == "switch_to_split"));
    }
}
