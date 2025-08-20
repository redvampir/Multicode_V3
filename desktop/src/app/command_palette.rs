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
];
