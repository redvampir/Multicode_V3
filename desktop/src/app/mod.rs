pub mod command_palette;
pub mod command_translations;
pub mod diff;
pub mod events;
pub mod io;
pub mod log_translations;
pub mod search_translations;
pub mod settings_translations;
pub mod ui;

mod actions;
mod state;
mod view;

pub use crate::visual::translations::Language;
pub use log_translations::{format_log, LogMessage};
pub use state::{
    AppTheme, CreateTarget, Diagnostic, EditorMode, EntryType, FileEntry, LogEntry, LogLevel,
    MulticodeApp, PendingAction, Screen, Tab, TabDragState, UserSettings, ViewMode,
};

use iced::Application;
use iced::Settings;
use std::path::PathBuf;

pub fn run(path: Option<PathBuf>) -> iced::Result {
    let settings = UserSettings::load();
    let flags = path.or_else(|| settings.last_folders.first().cloned());
    MulticodeApp::run(Settings {
        flags,
        ..Settings::default()
    })
}

/// Save log entries to a file in JSON format.
pub fn save_log_to_file<P: AsRef<std::path::Path>>(
    entries: &[LogEntry],
    path: P,
) -> Result<(), String> {
    use std::fs::File;
    let items: Vec<serde_json::Value> = entries
        .iter()
        .map(|e| {
            serde_json::json!({
                "level": e.level.to_string(),
                "message_key": format!("{:?}", e.message_key),
                "args": e.args,
            })
        })
        .collect();
    let file = File::create(path.as_ref()).map_err(|e| e.to_string())?;
    serde_json::to_writer_pretty(file, &items).map_err(|e| e.to_string())
}
