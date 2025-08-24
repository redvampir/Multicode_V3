use crate::visual::connections::Connection;
use chrono::{DateTime, Utc};
use directories::ProjectDirs;
use iced::{widget::text_editor, Color};
use multicode_core::{git, meta::VisualMeta, BlockInfo};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::ops::Range;
use std::path::PathBuf;
use tokio::{fs, process::Child, sync::broadcast};

use super::log_translations::LogMessage;
use crate::app::diff::DiffView;
use crate::components::file_manager::ContextMenu;
use crate::editor::{AutocompleteState, EditorSettings};
use crate::visual::palette::PaletteBlock;
use crate::visual::translations::Language;

mod serde_color {
    use iced::Color;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(color: &Color, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        [color.r, color.g, color.b].serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Color, D::Error>
    where
        D: Deserializer<'de>,
    {
        let [r, g, b] = <[f32; 3]>::deserialize(deserializer)?;
        Ok(Color::from_rgb(r, g, b))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Info => write!(f, "Info"),
            LogLevel::Warning => write!(f, "Warning"),
            LogLevel::Error => write!(f, "Error"),
        }
    }
}

impl LogLevel {
    pub const ALL: [LogLevel; 3] = [LogLevel::Info, LogLevel::Warning, LogLevel::Error];
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message_key: LogMessage,
    pub args: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

impl LogEntry {
    pub fn new(message_key: LogMessage, args: Vec<String>, timestamp: DateTime<Utc>) -> Self {
        let level = message_key.level();
        Self {
            level,
            message_key,
            args,
            timestamp,
        }
    }

    pub fn raw(message: String, timestamp: DateTime<Utc>) -> Self {
        Self {
            level: LogLevel::Info,
            message_key: LogMessage::Raw,
            args: vec![message],
            timestamp,
        }
    }
}

#[derive(Debug)]
pub struct MulticodeApp {
    pub(super) screen: Screen,
    pub(super) view_mode: ViewMode,
    pub(super) files: Vec<FileEntry>,
    pub(super) tabs: Vec<Tab>,
    /// индекс активной вкладки
    pub(super) active_tab: Option<usize>,
    /// строка поиска
    pub(super) search_term: String,
    /// строка замены
    pub(super) replace_term: String,
    /// найденные совпадения
    pub(super) search_results: Vec<(usize, Range<usize>)>,
    /// отображать панель поиска
    pub(super) show_search_panel: bool,
    /// текущий индекс совпадения
    pub(super) current_match: Option<usize>,
    /// имя для создания нового файла
    pub(super) new_file_name: String,
    /// имя для создания новой директории
    pub(super) new_directory_name: String,
    /// что создавать: файл или директорию
    pub(super) create_target: CreateTarget,
    /// новое имя при переименовании
    pub(super) rename_file_name: String,
    /// фильтр файлового менеджера
    pub(super) search_query: String,
    /// избранные файлы и директории
    pub(super) favorites: Vec<PathBuf>,
    pub(super) query: String,
    pub(super) show_command_palette: bool,
    pub(super) log: Vec<LogEntry>,
    /// минимальный уровень отображаемых записей журнала
    pub(super) min_log_level: LogLevel,
    /// результаты поиска по проекту
    pub(super) project_search_results: Vec<(PathBuf, usize, String)>,
    /// строка для перехода после открытия файла
    pub(super) goto_line: Option<usize>,
    /// текстовое значение для перехода к строке
    pub(super) goto_line_input: String,
    /// отображать модальное окно перехода к строке
    pub(super) show_goto_line_modal: bool,
    pub(super) show_terminal: bool,
    pub(super) terminal_cmd: String,
    pub(super) terminal_child: Option<Child>,
    pub(super) show_terminal_help: bool,
    pub(super) sender: broadcast::Sender<String>,
    pub(super) settings: UserSettings,
    pub(super) expanded_dirs: HashSet<PathBuf>,
    pub(super) context_menu: Option<ContextMenu>,
    pub(super) selected_path: Option<PathBuf>,
    /// отображать подтверждение перезаписи файла
    pub(super) show_create_file_confirm: bool,
    /// отображать подтверждение удаления файла
    pub(super) show_delete_confirm: bool,
    /// ожидаемое действие при подтверждении потери изменений
    pub(super) pending_action: Option<PendingAction>,
    pub(super) hotkey_capture: Option<HotkeyField>,
    pub(super) shortcut_capture: Option<String>,
    pub(super) settings_warning: Option<String>,
    pub(super) loading: bool,
    pub(super) diff_error: Option<String>,
    pub(super) show_meta_dialog: bool,
    pub(super) meta_tags: String,
    pub(super) meta_links: String,
    pub(super) meta_comment: String,
    pub(super) autocomplete: Option<AutocompleteState>,
    pub(super) show_meta_panel: bool,
    pub(super) tab_drag: Option<TabDragState>,
    pub(super) palette: Vec<PaletteBlock>,
    pub(super) palette_categories: Vec<(String, Vec<usize>)>,
    pub(super) show_block_palette: bool,
    pub(super) palette_query: String,
    pub(super) palette_drag: Option<BlockInfo>,
    /// history of executed commands
    pub(super) recent_commands: VecDeque<String>,
    /// usage count for executed commands
    pub(super) command_counts: HashMap<String, usize>,
}

#[derive(Debug, Clone)]
pub enum Screen {
    ProjectPicker,
    TextEditor { root: PathBuf },
    VisualEditor { root: PathBuf },
    Split { root: PathBuf },
    Diff(DiffView),
    Settings,
}

#[derive(Debug, Clone)]
pub enum PendingAction {
    Select(PathBuf),
    Delete(PathBuf),
}

#[derive(Debug, Clone)]
pub enum EntryType {
    File,
    Dir,
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub ty: EntryType,
    pub has_meta: bool,
    pub children: Vec<FileEntry>,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub line: usize,
    pub range: Range<usize>,
    pub message: String,
}

#[derive(Debug)]
pub struct Tab {
    pub path: PathBuf,
    pub content: String,
    pub editor: text_editor::Content,
    pub dirty: bool,
    pub blame: HashMap<usize, git::BlameLine>,
    pub diagnostics: Vec<Diagnostic>,
    pub blocks: Vec<BlockInfo>,
    pub connections: Vec<Connection>,
    pub meta: Option<VisualMeta>,
    pub undo_stack: VecDeque<String>,
    pub redo_stack: VecDeque<String>,
    pub analysis_version: u64,
}

#[derive(Debug)]
pub struct TabDragState {
    pub index: usize,
    pub start: f32,
    pub current: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewMode {
    Code,
    Schema,
    Split,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreateTarget {
    File,
    Directory,
}

impl CreateTarget {
    pub const ALL: [CreateTarget; 2] = [CreateTarget::File, CreateTarget::Directory];
}

impl ToString for CreateTarget {
    fn to_string(&self) -> String {
        match self {
            CreateTarget::File => "Файл".into(),
            CreateTarget::Directory => "Папка".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hotkey {
    pub key: String,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

impl Hotkey {
    pub fn matches(&self, key: &iced::keyboard::Key, modifiers: iced::keyboard::Modifiers) -> bool {
        self.ctrl == modifiers.control()
            && self.alt == modifiers.alt()
            && self.shift == modifiers.shift()
            && match key {
                iced::keyboard::Key::Character(c) => c.eq_ignore_ascii_case(&self.key),
                iced::keyboard::Key::Named(named) => {
                    self.key.eq_ignore_ascii_case(&format!("{:?}", named))
                }
                _ => false,
            }
    }
}

impl fmt::Display for Hotkey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        if self.ctrl {
            parts.push("Ctrl".to_string());
        }
        if self.alt {
            parts.push("Alt".to_string());
        }
        if self.shift {
            parts.push("Shift".to_string());
        }
        parts.push(self.key.clone());
        write!(f, "{}", parts.join("+"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Hotkeys {
    pub create_file: Hotkey,
    pub save_file: Hotkey,
    pub rename_file: Hotkey,
    pub delete_file: Hotkey,
    pub next_diff: Hotkey,
    pub prev_diff: Hotkey,
}

impl Default for Hotkeys {
    fn default() -> Self {
        Self {
            create_file: Hotkey {
                key: "N".into(),
                ctrl: true,
                alt: false,
                shift: false,
            },
            save_file: Hotkey {
                key: "S".into(),
                ctrl: true,
                alt: false,
                shift: false,
            },
            rename_file: Hotkey {
                key: "F2".into(),
                ctrl: false,
                alt: false,
                shift: false,
            },
            delete_file: Hotkey {
                key: "Delete".into(),
                ctrl: false,
                alt: false,
                shift: false,
            },
            next_diff: Hotkey {
                key: "F8".into(),
                ctrl: false,
                alt: false,
                shift: false,
            },
            prev_diff: Hotkey {
                key: "F7".into(),
                ctrl: false,
                alt: false,
                shift: false,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyField {
    CreateFile,
    SaveFile,
    RenameFile,
    DeleteFile,
    NextDiff,
    PrevDiff,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorMode {
    Text,
    Visual,
    Split,
}

impl Default for EditorMode {
    fn default() -> Self {
        EditorMode::Text
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppTheme {
    Light,
    Dark,
    HighContrast,
}

impl AppTheme {
    pub const ALL: [AppTheme; 3] = [AppTheme::Light, AppTheme::Dark, AppTheme::HighContrast];
}

impl Default for AppTheme {
    fn default() -> Self {
        AppTheme::Light
    }
}

impl fmt::Display for AppTheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppTheme::Light => write!(f, "Light"),
            AppTheme::Dark => write!(f, "Dark"),
            AppTheme::HighContrast => write!(f, "High Contrast"),
        }
    }
}

fn default_syntect_theme() -> String {
    "InspiredGitHub".into()
}

fn default_true() -> bool {
    true
}

fn default_view_mode() -> ViewMode {
    ViewMode::Code
}

fn default_match_color() -> Color {
    Color::from_rgb(1.0, 1.0, 0.0)
}

fn default_diagnostic_color() -> Color {
    Color::from_rgb(1.0, 0.0, 0.0)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    #[serde(default)]
    pub last_folders: Vec<PathBuf>,
    #[serde(default)]
    pub default_entry: Option<PathBuf>,
    #[serde(default)]
    pub hotkeys: Hotkeys,
    #[serde(default)]
    pub shortcuts: HashMap<String, Hotkey>,
    #[serde(default)]
    pub editor_mode: EditorMode,
    #[serde(default = "default_view_mode")]
    pub last_view_mode: ViewMode,
    #[serde(default)]
    pub theme: AppTheme,
    #[serde(default = "default_syntect_theme")]
    pub syntect_theme: String,
    #[serde(default)]
    pub editor: EditorSettings,
    #[serde(default = "default_match_color", with = "serde_color")]
    pub match_color: Color,
    #[serde(default = "default_diagnostic_color", with = "serde_color")]
    pub diagnostic_color: Color,
    #[serde(default)]
    pub language: Language,
    #[serde(default)]
    pub show_line_numbers: bool,
    #[serde(default)]
    pub show_status_bar: bool,
    #[serde(default = "default_true")]
    pub show_toolbar: bool,
    #[serde(default)]
    pub show_markdown_preview: bool,
    #[serde(default)]
    pub favorites: Vec<PathBuf>,
    #[serde(default)]
    pub block_favorites: Vec<String>,
    #[serde(default)]
    pub recent_commands: Vec<String>,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            last_folders: Vec::new(),
            default_entry: None,
            hotkeys: Hotkeys::default(),
            shortcuts: HashMap::new(),
            editor_mode: EditorMode::Text,
            last_view_mode: ViewMode::Code,
            theme: AppTheme::default(),
            syntect_theme: default_syntect_theme(),
            editor: EditorSettings::default(),
            match_color: default_match_color(),
            diagnostic_color: default_diagnostic_color(),
            language: Language::default(),
            show_line_numbers: true,
            show_status_bar: true,
            show_toolbar: true,
            show_markdown_preview: false,
            favorites: Vec::new(),
            block_favorites: Vec::new(),
            recent_commands: Vec::new(),
        }
    }
}

impl UserSettings {
    pub const MAX_RECENT: usize = 5;

    pub fn add_recent_folder(&mut self, path: PathBuf) {
        self.last_folders.retain(|p| p != &path);
        self.last_folders.insert(0, path);
        if self.last_folders.len() > Self::MAX_RECENT {
            self.last_folders.truncate(Self::MAX_RECENT);
        }
    }

    pub fn load() -> Self {
        tokio::runtime::Runtime::new()
            .map(|rt| rt.block_on(Self::load_async()))
            .unwrap_or_default()
    }

    async fn load_async() -> Self {
        if let Some(proj) = ProjectDirs::from("com", "multicode", "multicode") {
            let path = proj.config_dir().join("settings.json");
            if let Ok(data) = fs::read_to_string(path).await {
                if let Ok(s) = serde_json::from_str(&data) {
                    return s;
                }
            }
        }
        Self::default()
    }

    pub async fn save(self) {
        if let Some(proj) = ProjectDirs::from("com", "multicode", "multicode") {
            let path = proj.config_dir().join("settings.json");
            let _ = fs::create_dir_all(path.parent().unwrap()).await;
            if let Ok(json) = serde_json::to_string_pretty(&self) {
                let _ = fs::write(path, json).await;
            }
        }
    }
}

impl MulticodeApp {
    pub fn view_mode(&self) -> ViewMode {
        self.view_mode
    }

    pub fn current_file(&self) -> Option<&Tab> {
        self.active_tab.and_then(|i| self.tabs.get(i))
    }

    pub fn current_file_mut(&mut self) -> Option<&mut Tab> {
        if let Some(i) = self.active_tab {
            self.tabs.get_mut(i)
        } else {
            None
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.current_file().map(|f| f.dirty).unwrap_or(false)
    }

    pub fn set_dirty(&mut self, value: bool) {
        if let Some(f) = self.current_file_mut() {
            f.dirty = value;
        }
    }

    pub fn is_visual_mode(&self) -> bool {
        matches!(
            self.screen,
            Screen::VisualEditor { .. } | Screen::Split { .. }
        )
    }

    /// Возвращает путь к корню проекта, если он выбран
    pub fn current_root_path(&self) -> Option<PathBuf> {
        match &self.screen {
            Screen::TextEditor { root }
            | Screen::VisualEditor { root }
            | Screen::Split { root } => Some(root.clone()),
            Screen::Diff(_) => self.settings.last_folders.first().cloned(),
            Screen::ProjectPicker => None,
            Screen::Settings => self.settings.last_folders.first().cloned(),
        }
    }

    /// Строковое представление корневого каталога
    pub fn current_root(&self) -> String {
        self.current_root_path()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default()
    }

    pub fn search_results(&self) -> &[(usize, Range<usize>)] {
        &self.search_results
    }

    pub fn autocomplete(&self) -> Option<&AutocompleteState> {
        self.autocomplete.as_ref()
    }

    pub fn settings(&self) -> &UserSettings {
        &self.settings
    }

    pub fn show_meta_panel(&self) -> bool {
        self.show_meta_panel
    }
}

impl Drop for MulticodeApp {
    fn drop(&mut self) {
        self.settings.last_view_mode = self.view_mode;
        if let Ok(rt) = tokio::runtime::Runtime::new() {
            let settings = self.settings.clone();
            rt.block_on(settings.save());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::events::Message;
    use tokio::sync::broadcast;
    use std::collections::{HashMap, HashSet, VecDeque};

    fn build_app() -> MulticodeApp {
        let (sender, _) = broadcast::channel(1);
        MulticodeApp {
            screen: Screen::ProjectPicker,
            view_mode: ViewMode::Code,
            files: Vec::new(),
            tabs: Vec::new(),
            active_tab: None,
            search_term: String::new(),
            replace_term: String::new(),
            search_results: Vec::new(),
            show_search_panel: false,
            current_match: None,
            new_file_name: String::new(),
            new_directory_name: String::new(),
            create_target: CreateTarget::File,
            rename_file_name: String::new(),
            search_query: String::new(),
            favorites: Vec::new(),
            query: String::new(),
            show_command_palette: false,
            log: Vec::new(),
            min_log_level: LogLevel::Info,
            project_search_results: Vec::new(),
            goto_line: None,
            goto_line_input: String::new(),
            show_goto_line_modal: false,
            show_terminal: false,
            terminal_cmd: String::new(),
            terminal_child: None,
            show_terminal_help: false,
            sender,
            settings: UserSettings::default(),
            expanded_dirs: HashSet::new(),
            context_menu: None,
            selected_path: None,
            show_create_file_confirm: false,
            show_delete_confirm: false,
            pending_action: None,
            hotkey_capture: None,
            shortcut_capture: None,
            settings_warning: None,
            loading: false,
            diff_error: None,
            show_meta_dialog: false,
            meta_tags: String::new(),
            meta_links: String::new(),
            meta_comment: String::new(),
            autocomplete: None,
            show_meta_panel: false,
            tab_drag: None,
            palette: Vec::new(),
            palette_categories: Vec::new(),
            show_block_palette: false,
            palette_query: String::new(),
            palette_drag: None,
            recent_commands: VecDeque::new(),
            command_counts: HashMap::new(),
        }
    }

    #[test]
    fn execute_command_updates_history() {
        let mut app = build_app();
        let _ = app.handle_message(Message::ExecuteCommand("test_cmd".into()));
        assert_eq!(app.recent_commands, VecDeque::from(vec!["test_cmd".to_string()]));
        assert_eq!(app.command_counts.get("test_cmd"), Some(&1));
    }

    #[test]
    fn recent_commands_are_limited_and_counts_updated() {
        let mut app = build_app();
        for i in 0..60 {
            let cmd = if i < 55 { "a" } else { "b" };
            let _ = app.handle_message(Message::ExecuteCommand(cmd.into()));
        }
        assert_eq!(app.recent_commands.len(), 50);
        assert_eq!(app.command_counts.get("a"), Some(&45));
        assert_eq!(app.command_counts.get("b"), Some(&5));
    }

    #[test]
    fn user_settings_serialization_roundtrip_preserves_recent_commands() {
        let mut settings = UserSettings::default();
        settings.recent_commands = vec!["cmd1".into(), "cmd2".into()];
        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: UserSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.recent_commands, settings.recent_commands);
    }

    #[test]
    fn user_settings_deserialization_defaults_recent_commands() {
        let json = "{}";
        let settings: UserSettings = serde_json::from_str(json).unwrap();
        assert!(settings.recent_commands.is_empty());
    }
}
