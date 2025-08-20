pub mod command_palette;
pub mod diff;
pub mod events;
pub mod io;
pub mod ui;

mod actions;
mod state;
mod view;

pub use state::{
    AppTheme, CreateTarget, Diagnostic, EditorMode, EntryType, FileEntry, Hotkey, HotkeyField,
    Hotkeys, Language, MulticodeApp, PendingAction, Screen, Tab, TabDragState, UserSettings,
};

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
