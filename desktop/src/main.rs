mod modal;

use iced::futures::stream;
#[allow(unused_imports)]
use iced::widget::overlay::menu as menu;
use iced::widget::{
    button, column, container, pick_list, row, scrollable, text, text_editor, text_input,
    MouseArea, Space,
};
use crate::modal::Modal;
use iced::{
    alignment, event, keyboard, subscription, Application, Command, Element, Event, Length,
    Settings, Subscription, Theme,
};
use multicode_core::{blocks, export, git, search};
use tokio::{fs, sync::broadcast};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;
use std::path::{Path, PathBuf};

pub fn main() -> iced::Result {
    MulticodeApp::run(Settings::default())
}

#[derive(Debug)]
struct MulticodeApp {
    screen: Screen,
    files: Vec<FileEntry>,
    /// выбранный в данный момент файл
    selected_file: Option<PathBuf>,
    /// содержимое открытого файла
    file_content: String,
    /// состояние текстового редактора
    editor: text_editor::Content,
    /// имя для создания нового файла
    new_file_name: String,
    /// имя для создания нового каталога
    new_folder_name: String,
    /// новое имя при переименовании
    rename_file_name: String,
    query: String,
    log: Vec<String>,
    sender: broadcast::Sender<String>,
    settings: UserSettings,
    expanded_dirs: HashSet<PathBuf>,
    context_menu: Option<ContextMenu>,
    /// отображать подтверждение перезаписи файла
    show_create_file_confirm: bool,
    /// отображать подтверждение удаления файла
    show_delete_confirm: bool,
    /// есть ли несохранённые изменения
    dirty: bool,
    /// ожидаемое действие при подтверждении потери изменений
    pending_action: Option<PendingAction>,
    hotkey_capture: Option<HotkeyField>,
    settings_warning: Option<String>,
}

#[derive(Debug, Clone)]
enum Screen {
    ProjectPicker,
    TextEditor { root: PathBuf },
    VisualEditor { root: PathBuf },
    Settings,
}

#[derive(Debug, Clone)]
enum Message {
    PickFolder,
    FolderPicked(Option<PathBuf>),
    FilesLoaded(Vec<FileEntry>),
    QueryChanged(String),
    // выбор файла и операции над ним
    SelectFile(PathBuf),
    FileLoaded(Result<(PathBuf, String), String>),
    FileContentEdited(text_editor::Action),
    SaveFile,
    FileSaved(Result<(), String>),
    NewFileNameChanged(String),
    NewFolderNameChanged(String),
    CreateFile,
    /// подтверждение создания при наличии файла
    ConfirmCreateFile,
    /// отмена создания при наличии файла
    CancelCreateFile,
    FileCreated(Result<PathBuf, String>),
    CreateFolder,
    FolderCreated(Result<PathBuf, String>),
    RenameFileNameChanged(String),
    RenameFile,
    FileRenamed(Result<PathBuf, String>),
    /// запрос на удаление выбранного файла
    RequestDeleteFile,
    /// подтверждение удаления файла
    DeleteFile,
    /// отмена удаления файла
    CancelDeleteFile,
    FileDeleted(Result<PathBuf, String>),
    RunSearch,
    SearchFinished(Result<Vec<String>, String>),
    RunParse,
    ParseFinished(Result<Vec<String>, String>),
    RunGitLog,
    GitFinished(Result<Vec<String>, String>),
    RunExport,
    ExportFinished(Result<Vec<String>, String>),
    CoreEvent(String),
    IcedEvent(Event),
    SaveSettings,
    SettingsSaved,
    OpenSettings,
    CloseSettings,
    ToggleDir(PathBuf),
    ShowContextMenu(PathBuf),
    CloseContextMenu,
    /// подтверждение потери несохранённых изменений
    ConfirmDiscard,
    /// отмена потери несохранённых изменений
    CancelDiscard,
    ThemeSelected(AppTheme),
    LanguageSelected(Language),
    StartCaptureHotkey(HotkeyField),
    SwitchToTextEditor,
    SwitchToVisualEditor,
}

#[derive(Debug, Clone)]
enum PendingAction {
    Select(PathBuf),
    Delete(PathBuf),
}

#[derive(Debug, Clone)]
enum EntryType {
    File,
    Dir,
}

#[derive(Debug, Clone)]
struct FileEntry {
    path: PathBuf,
    ty: EntryType,
    children: Vec<FileEntry>,
}

#[derive(Debug)]
struct ContextMenu {
    path: PathBuf,
    state: std::cell::RefCell<menu::State>,
    hovered: std::cell::RefCell<Option<usize>>,
}

impl ContextMenu {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            state: std::cell::RefCell::new(menu::State::new()),
            hovered: std::cell::RefCell::new(None),
        }
    }
}

#[derive(Debug, Clone)]
enum ContextMenuItem {
    Open,
    Rename,
    Delete,
}

impl ToString for ContextMenuItem {
    fn to_string(&self) -> String {
        match self {
            ContextMenuItem::Open => "Открыть".into(),
            ContextMenuItem::Rename => "Переименовать".into(),
            ContextMenuItem::Delete => "Удалить".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Hotkey {
    key: String,
    ctrl: bool,
    alt: bool,
    shift: bool,
}

impl Hotkey {
    fn matches(&self, key: &keyboard::Key, modifiers: keyboard::Modifiers) -> bool {
        self.ctrl == modifiers.control()
            && self.alt == modifiers.alt()
            && self.shift == modifiers.shift()
            && match key {
                keyboard::Key::Character(c) => c.eq_ignore_ascii_case(&self.key),
                keyboard::Key::Named(named) => {
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
struct Hotkeys {
    create_file: Hotkey,
    save_file: Hotkey,
    rename_file: Hotkey,
    delete_file: Hotkey,
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
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HotkeyField {
    CreateFile,
    SaveFile,
    RenameFile,
    DeleteFile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum EditorMode {
    Text,
    Visual,
}

impl Default for EditorMode {
    fn default() -> Self {
        EditorMode::Text
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum AppTheme {
    Light,
    Dark,
}

impl AppTheme {
    const ALL: [AppTheme; 2] = [AppTheme::Light, AppTheme::Dark];
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
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum Language {
    English,
    Russian,
}

impl Language {
    const ALL: [Language; 2] = [Language::English, Language::Russian];
}

impl Default for Language {
    fn default() -> Self {
        Language::English
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Language::English => write!(f, "English"),
            Language::Russian => write!(f, "Русский"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserSettings {
    last_folder: Option<PathBuf>,
    #[serde(default)]
    hotkeys: Hotkeys,
    #[serde(default)]
    editor_mode: EditorMode,
    #[serde(default)]
    theme: AppTheme,
    #[serde(default)]
    language: Language,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            last_folder: None,
            hotkeys: Hotkeys::default(),
            editor_mode: EditorMode::Text,
            theme: AppTheme::default(),
            language: Language::default(),
        }
    }
}

impl UserSettings {
    fn load() -> Self {
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

    async fn save(self) {
        if let Some(proj) = ProjectDirs::from("com", "multicode", "multicode") {
            let path = proj.config_dir().join("settings.json");
            let _ = fs::create_dir_all(path.parent().unwrap()).await;
            if let Ok(json) = serde_json::to_string_pretty(&self) {
                let _ = fs::write(path, json).await;
            }
        }
    }
}

impl Application for MulticodeApp {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let settings = UserSettings::load();
        let (sender, _) = broadcast::channel(100);

        let app = MulticodeApp {
            screen: if let Some(path) = settings.last_folder.clone() {
                match settings.editor_mode {
                    EditorMode::Text => Screen::TextEditor { root: path },
                    EditorMode::Visual => Screen::VisualEditor { root: path },
                }
            } else {
                Screen::ProjectPicker
            },
            files: Vec::new(),
            selected_file: None,
            file_content: String::new(),
            editor: text_editor::Content::new(),
            new_file_name: String::new(),
            new_folder_name: String::new(),
            rename_file_name: String::new(),
            query: String::new(),
            log: Vec::new(),
            sender,
            settings,
            expanded_dirs: HashSet::new(),
            context_menu: None,
            show_create_file_confirm: false,
            show_delete_confirm: false,
            dirty: false,
            pending_action: None,
            hotkey_capture: None,
            settings_warning: None,
        };

        let cmd = match &app.screen {
            Screen::TextEditor { root } | Screen::VisualEditor { root } => {
                app.load_files(root.clone())
            }
            _ => Command::none(),
        };

        (app, cmd)
    }

    fn title(&self) -> String {
        String::from("Multicode Desktop")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::IcedEvent(Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                modifiers,
                ..
            })) => {
                if let Some(field) = self.hotkey_capture.take() {
                    let key_str = match key {
                        keyboard::Key::Character(c) => c.to_string().to_uppercase(),
                        keyboard::Key::Named(named) => format!("{:?}", named),
                        _ => return Command::none(),
                    };
                    let hk = Hotkey {
                        key: key_str,
                        ctrl: modifiers.control(),
                        alt: modifiers.alt(),
                        shift: modifiers.shift(),
                    };
                    match field {
                        HotkeyField::CreateFile => self.settings.hotkeys.create_file = hk,
                        HotkeyField::SaveFile => self.settings.hotkeys.save_file = hk,
                        HotkeyField::RenameFile => self.settings.hotkeys.rename_file = hk,
                        HotkeyField::DeleteFile => self.settings.hotkeys.delete_file = hk,
                    }
                    return Command::none();
                }
                if !matches!(
                    self.screen,
                    Screen::TextEditor { .. } | Screen::VisualEditor { .. }
                ) {
                    return Command::none();
                }
                let hotkeys = &self.settings.hotkeys;
                if hotkeys.create_file.matches(&key, modifiers) {
                    return self.update(Message::CreateFile);
                }
                if hotkeys.save_file.matches(&key, modifiers) {
                    return self.update(Message::SaveFile);
                }
                if hotkeys.rename_file.matches(&key, modifiers) {
                    return self.update(Message::RenameFile);
                }
                if hotkeys.delete_file.matches(&key, modifiers) {
                    return self.update(Message::RequestDeleteFile);
                }
                Command::none()
            }
            Message::IcedEvent(_) => Command::none(),
            Message::ThemeSelected(theme) => {
                self.settings.theme = theme;
                Command::none()
            }
            Message::LanguageSelected(lang) => {
                self.settings.language = lang;
                Command::none()
            }
            Message::StartCaptureHotkey(field) => {
                self.hotkey_capture = Some(field);
                Command::none()
            }
            Message::OpenSettings => {
                self.screen = Screen::Settings;
                Command::none()
            }
            Message::CloseSettings => {
                if let Some(root) = self.settings.last_folder.clone() {
                    self.screen = match self.settings.editor_mode {
                        EditorMode::Text => Screen::TextEditor { root },
                        EditorMode::Visual => Screen::VisualEditor { root },
                    };
                } else {
                    self.screen = Screen::ProjectPicker;
                }
                Command::none()
            }
            Message::SwitchToTextEditor => {
                if let Some(root) = self.current_root_path() {
                    self.screen = Screen::TextEditor { root: root.clone() };
                    self.settings.editor_mode = EditorMode::Text;
                    return self.update(Message::SaveSettings);
                }
                Command::none()
            }
            Message::SwitchToVisualEditor => {
                if let Some(root) = self.current_root_path() {
                    self.screen = Screen::VisualEditor { root: root.clone() };
                    self.settings.editor_mode = EditorMode::Visual;
                    return self.update(Message::SaveSettings);
                }
                Command::none()
            }
            Message::PickFolder => Command::perform(pick_folder(), Message::FolderPicked),
            Message::FolderPicked(path) => {
                if let Some(root) = path {
                    self.settings.last_folder = Some(root.clone());
                    self.screen = match self.settings.editor_mode {
                        EditorMode::Text => Screen::TextEditor { root: root.clone() },
                        EditorMode::Visual => Screen::VisualEditor { root: root.clone() },
                    };
                    multicode_core::meta::watch::spawn(self.sender.clone());
                    return Command::batch([
                        self.load_files(root),
                        self.update(Message::SaveSettings),
                    ]);
                }
                Command::none()
            }
            Message::FilesLoaded(list) => {
                self.files = list;
                Command::none()
            }
            Message::QueryChanged(q) => {
                self.query = q;
                Command::none()
            }
            Message::SelectFile(path) => {
                self.context_menu = None;
                if self.dirty && Some(path.clone()) != self.selected_file {
                    self.pending_action = Some(PendingAction::Select(path));
                    return Command::none();
                }
                return Command::perform(
                    async move {
                        match fs::read_to_string(&path).await {
                            Ok(c) => Ok((path, c)),
                            Err(e) => Err(format!("{}", e)),
                        }
                    },
                    Message::FileLoaded,
                );
            }
            Message::FileLoaded(Ok((path, content))) => {
                self.selected_file = Some(path);
                self.file_content = content;
                self.editor = text_editor::Content::with_text(&self.file_content);
                self.rename_file_name.clear();
                self.dirty = false;
                Command::none()
            }
            Message::FileLoaded(Err(e)) => {
                self.log.push(format!("ошибка чтения: {e}"));
                Command::none()
            }
            Message::FileContentEdited(action) => {
                self.editor.perform(action);
                self.file_content = self.editor.text();
                self.dirty = true;
                Command::none()
            }
            Message::SaveFile => {
                if let Some(path) = self.selected_file.clone() {
                    let content = self.file_content.clone();
                    return Command::perform(
                        async move {
                            fs::write(&path, content)
                                .await
                                .map_err(|e| format!("{}", e))
                        },
                        Message::FileSaved,
                    );
                }
                Command::none()
            }
            Message::FileSaved(Ok(())) => {
                self.log.push("файл сохранен".into());
                self.dirty = false;
                Command::none()
            }
            Message::FileSaved(Err(e)) => {
                self.log.push(format!("ошибка сохранения: {e}"));
                Command::none()
            }
            Message::NewFileNameChanged(s) => {
                self.new_file_name = s;
                Command::none()
            }
            Message::NewFolderNameChanged(s) => {
                self.new_folder_name = s;
                Command::none()
            }
            Message::CreateFile => {
                if let Some(root) = self.current_root_path() {
                    let name = self.new_file_name.clone();
                    if name.is_empty() {
                        self.log.push("имя файла не задано".into());
                        return Command::none();
                    }
                    let path = root.join(&name);
                    if path.exists() {
                        self.log.push(format!("{} уже существует", path.display()));
                        self.show_create_file_confirm = true;
                        return Command::none();
                    }
                    return Command::perform(
                        async move {
                            std::fs::OpenOptions::new()
                                .write(true)
                                .create_new(true)
                                .open(&path)
                                .map(|_| path)
                                .map_err(|e| format!("{}", e))
                        },
                        Message::FileCreated,
                    );
                }
                Command::none()
            }
            Message::ConfirmCreateFile => {
                if let Some(root) = self.current_root_path() {
                    let path = root.join(&self.new_file_name);
                    self.show_create_file_confirm = false;
                    return Command::perform(
                        async move {
                            let _ = std::fs::remove_file(&path);
                            std::fs::OpenOptions::new()
                                .write(true)
                                .create_new(true)
                                .open(&path)
                                .map(|_| path)
                                .map_err(|e| format!("{}", e))
                        },
                        Message::FileCreated,
                    );
                }
                Command::none()
            }
            Message::CancelCreateFile => {
                self.show_create_file_confirm = false;
                Command::none()
            }
            Message::FileCreated(Ok(path)) => {
                self.log.push(format!("создан {}", path.display()));
                self.new_file_name.clear();
                self.selected_file = Some(path.clone());
                self.file_content.clear();
                self.editor = text_editor::Content::new();
                return self.load_files(self.current_root_path().unwrap());
            }
            Message::FileCreated(Err(e)) => {
                self.log.push(format!("ошибка создания: {e}"));
                Command::none()
            }
            Message::CreateFolder => {
                if let Some(root) = self.current_root_path() {
                    let name = self.new_folder_name.clone();
                    if name.is_empty() {
                        self.log.push("имя каталога не задано".into());
                        return Command::none();
                    }
                    let path = root.join(&name);
                    return Command::perform(
                        async move {
                            fs::create_dir(&path)
                                .await
                                .map(|_| path)
                                .map_err(|e| format!("{}", e))
                        },
                        Message::FolderCreated,
                    );
                }
                Command::none()
            }
            Message::FolderCreated(Ok(path)) => {
                self.log.push(format!("создан каталог {}", path.display()));
                self.new_folder_name.clear();
                return self.load_files(self.current_root_path().unwrap());
            }
            Message::FolderCreated(Err(e)) => {
                self.log.push(format!("ошибка создания каталога: {e}"));
                Command::none()
            }
            Message::RenameFileNameChanged(s) => {
                self.rename_file_name = s;
                Command::none()
            }
            Message::RenameFile => {
                self.context_menu = None;
                if let Some(old_path) = self.selected_file.clone() {
                    let new_name = self.rename_file_name.clone();
                    if new_name.is_empty() {
                        self.log.push("новое имя пустое".into());
                        return Command::none();
                    }
                    let new_path = old_path.parent().unwrap().join(new_name);
                    return Command::perform(
                        async move {
                            fs::rename(&old_path, &new_path)
                                .await
                                .map(|_| new_path)
                                .map_err(|e| format!("{}", e))
                        },
                        Message::FileRenamed,
                    );
                }
                Command::none()
            }
            Message::FileRenamed(Ok(path)) => {
                self.log.push(format!("переименовано в {}", path.display()));
                self.selected_file = Some(path);
                self.rename_file_name.clear();
                return self.load_files(self.current_root_path().unwrap());
            }
            Message::FileRenamed(Err(e)) => {
                self.log.push(format!("ошибка переименования: {e}"));
                Command::none()
            }
            Message::RequestDeleteFile => {
                if self.selected_file.is_some() {
                    self.show_delete_confirm = true;
                }
                Command::none()
            }
            Message::CancelDeleteFile => {
                self.show_delete_confirm = false;
                Command::none()
            }
            Message::DeleteFile => {
                self.context_menu = None;
                self.show_delete_confirm = false;
                if let Some(path) = self.selected_file.clone() {
                    if self.dirty {
                        self.pending_action = Some(PendingAction::Delete(path));
                        return Command::none();
                    }
                    return Command::perform(
                        async move {
                            fs::remove_file(&path)
                                .await
                                .map(|_| path)
                                .map_err(|e| format!("{}", e))
                        },
                        Message::FileDeleted,
                    );
                }
                Command::none()
            }
            Message::FileDeleted(Ok(path)) => {
                self.log.push(format!("удален {}", path.display()));
                self.selected_file = None;
                self.file_content.clear();
                self.editor = text_editor::Content::new();
                self.dirty = false;
                return self.load_files(self.current_root_path().unwrap());
            }
            Message::FileDeleted(Err(e)) => {
                self.log.push(format!("ошибка удаления: {e}"));
                Command::none()
            }
            Message::RunSearch => {
                let root = self.current_root();
                let query = self.query.clone();
                Command::perform(
                    async move {
                        let results = search::search_metadata(Path::new(&root), &query);
                        Ok::<_, String>(
                            results
                                .into_iter()
                                .map(|r| r.file.display().to_string())
                                .collect(),
                        )
                    },
                    |r| Message::SearchFinished(r),
                )
            }
            Message::SearchFinished(Ok(list)) => {
                for item in list {
                    self.log.push(format!("найден {item}"));
                }
                Command::none()
            }
            Message::SearchFinished(Err(e)) => {
                self.log.push(format!("ошибка поиска: {e}"));
                Command::none()
            }
            Message::RunParse => {
                let files = self.file_paths();
                Command::perform(
                    async move {
                        let mut lines = Vec::new();
                        for path in files {
                            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                            let lang = match ext {
                                "rs" => "rust",
                                "py" => "python",
                                "js" => "javascript",
                                "css" => "css",
                                "html" => "html",
                                _ => {
                                    lines.push(format!("{}: неизвестный язык", path.display()));
                                    continue;
                                }
                            };
                            match fs::read_to_string(&path).await {
                                Ok(content) => {
                                    match blocks::parse_blocks(content, lang.to_string()) {
                                        Some(b) => lines.push(format!(
                                            "{}: блоков {}",
                                            path.display(),
                                            b.len()
                                        )),
                                        None => lines.push(format!(
                                            "{}: не удалось разобрать",
                                            path.display()
                                        )),
                                    }
                                }
                                Err(e) => {
                                    lines.push(format!("{}: ошибка чтения {e}", path.display()))
                                }
                            }
                        }
                        Ok::<_, String>(lines)
                    },
                    Message::ParseFinished,
                )
            }
            Message::ParseFinished(Ok(lines)) => {
                self.log.extend(lines);
                Command::none()
            }
            Message::ParseFinished(Err(e)) => {
                self.log.push(format!("ошибка разбора: {e}"));
                Command::none()
            }
            Message::RunGitLog => Command::perform(
                async move { git::log().map_err(|e| e.to_string()) },
                Message::GitFinished,
            ),
            Message::GitFinished(Ok(lines)) => {
                self.log.extend(lines);
                Command::none()
            }
            Message::GitFinished(Err(e)) => {
                self.log.push(format!("ошибка git: {e}"));
                Command::none()
            }
            Message::RunExport => {
                let files = self.file_paths();
                Command::perform(
                    async move {
                        let mut lines = Vec::new();
                        for path in files {
                            match fs::read_to_string(&path).await {
                                Ok(content) => match export::serialize_viz_document(&content) {
                                    Some(json) => lines.push(format!("{}: {json}", path.display())),
                                    None => lines
                                        .push(format!("{}: метаданных не найдено", path.display())),
                                },
                                Err(e) => {
                                    lines.push(format!("{}: ошибка чтения {e}", path.display()))
                                }
                            }
                        }
                        Ok::<_, String>(lines)
                    },
                    Message::ExportFinished,
                )
            }
            Message::ExportFinished(Ok(lines)) => {
                self.log.extend(lines);
                Command::none()
            }
            Message::ExportFinished(Err(e)) => {
                self.log.push(format!("ошибка экспорта: {e}"));
                Command::none()
            }
            Message::ToggleDir(path) => {
                if !self.expanded_dirs.remove(&path) {
                    self.expanded_dirs.insert(path);
                }
                Command::none()
            }
            Message::ShowContextMenu(path) => {
                self.selected_file = Some(path.clone());
                self.context_menu = Some(ContextMenu::new(path));
                Command::none()
            }
            Message::CloseContextMenu => {
                self.context_menu = None;
                Command::none()
            }
            Message::ConfirmDiscard => {
                self.dirty = false;
                if let Some(action) = self.pending_action.take() {
                    match action {
                        PendingAction::Select(path) => {
                            return Command::perform(
                                async move {
                                    match fs::read_to_string(&path).await {
                                        Ok(c) => Ok((path, c)),
                                        Err(e) => Err(format!("{}", e)),
                                    }
                                },
                                Message::FileLoaded,
                            );
                        }
                        PendingAction::Delete(path) => {
                            return Command::perform(
                                async move {
                                    fs::remove_file(&path)
                                        .await
                                        .map(|_| path)
                                        .map_err(|e| format!("{}", e))
                                },
                                Message::FileDeleted,
                            );
                        }
                    }
                }
                Command::none()
            }
            Message::CancelDiscard => {
                self.pending_action = None;
                Command::none()
            }
            Message::CoreEvent(ev) => {
                self.log.push(ev);
                Command::none()
            }
            Message::SaveSettings => {
                let h = &self.settings.hotkeys;
                let mut set = HashSet::new();
                if !set.insert(h.create_file.to_string())
                    || !set.insert(h.save_file.to_string())
                    || !set.insert(h.rename_file.to_string())
                    || !set.insert(h.delete_file.to_string())
                {
                    self.settings_warning = Some("Сочетания клавиш должны быть уникальными".into());
                    return Command::none();
                }
                self.settings_warning = None;
                Command::perform(self.settings.clone().save(), |_| Message::SettingsSaved)
            }
            Message::SettingsSaved => Command::none(),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        if matches!(
            self.screen,
            Screen::TextEditor { .. } | Screen::VisualEditor { .. }
        ) {
            let rx = self.sender.subscribe();
            let core = subscription::run_with_id(
                "core-events",
                stream::unfold(rx, |mut rx| async {
                    match rx.recv().await {
                        Ok(ev) => Some((Message::CoreEvent(ev), rx)),
                        Err(_) => None,
                    }
                }),
            );
            let events = event::listen().map(Message::IcedEvent);
            Subscription::batch([core, events])
        } else {
            Subscription::none()
        }
    }

    fn theme(&self) -> Theme {
        match self.settings.theme {
            AppTheme::Light => Theme::Light,
            AppTheme::Dark => Theme::Dark,
        }
    }

    fn view(&self) -> Element<Message> {
        match &self.screen {
            Screen::ProjectPicker => {
                let settings_label = if self.settings.language == Language::Russian {
                    "Настройки"
                } else {
                    "Settings"
                };
                let content = column![
                    text("Выберите папку проекта"),
                    button("Выбрать папку").on_press(Message::PickFolder),
                    button(settings_label).on_press(Message::OpenSettings),
                ]
                .align_items(alignment::Alignment::Center)
                .spacing(20);

                container(content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x()
                    .center_y()
                    .into()
            }
            Screen::TextEditor { .. } => {
                let settings_label = if self.settings.language == Language::Russian {
                    "Настройки"
                } else {
                    "Settings"
                };
                let menu = row![
                    button("Разбор").on_press(Message::RunParse),
                    button("Поиск").on_press(Message::RunSearch),
                    button("Журнал Git").on_press(Message::RunGitLog),
                    button("Экспорт").on_press(Message::RunExport),
                    button(settings_label).on_press(Message::OpenSettings),
                ]
                .spacing(10);

                let sidebar = container(self.file_tree()).width(200);

                let rename_btn: Element<_> = if self.selected_file.is_some() {
                    button("Переименовать").on_press(Message::RenameFile).into()
                } else {
                    button("Переименовать").into()
                };
                let delete_btn: Element<_> = if self.selected_file.is_some() {
                    button("Удалить")
                        .on_press(Message::RequestDeleteFile)
                        .into()
                } else {
                    button("Удалить").into()
                };
                let save_label = if self.dirty {
                    "Сохранить*"
                } else {
                    "Сохранить"
                };
                let save_btn: Element<_> = if self.selected_file.is_some() {
                    button(save_label).on_press(Message::SaveFile).into()
                } else {
                    button(save_label).into()
                };
                let text_btn: Element<_> = button("Text").into();
                let visual_btn: Element<_> = button("Visual")
                    .on_press(Message::SwitchToVisualEditor)
                    .into();
                let mode_bar = row![text_btn, visual_btn, save_btn].spacing(5);

                let file_menu = row![
                    text_input("новый файл", &self.new_file_name)
                        .on_input(Message::NewFileNameChanged),
                    button("Создать файл").on_press(Message::CreateFile),
                    text_input("новый каталог", &self.new_folder_name)
                        .on_input(Message::NewFolderNameChanged),
                    button("Создать папку").on_press(Message::CreateFolder),
                    text_input("новое имя", &self.rename_file_name)
                        .on_input(Message::RenameFileNameChanged),
                    rename_btn,
                    delete_btn,
                ]
                .spacing(5);

                let warning: Element<_> = if self.show_create_file_confirm {
                    row![
                        text("Файл уже существует. Перезаписать?"),
                        button("Да").on_press(Message::ConfirmCreateFile),
                        button("Нет").on_press(Message::CancelCreateFile)
                    ]
                    .spacing(5)
                    .into()
                } else {
                    Space::with_width(Length::Shrink).into()
                };

                let dirty_warning: Element<_> = if self.pending_action.is_some() {
                    row![
                        text("Есть несохранённые изменения. Продолжить?"),
                        button("Да").on_press(Message::ConfirmDiscard),
                        button("Нет").on_press(Message::CancelDiscard)
                    ]
                    .spacing(5)
                    .into()
                } else {
                    Space::with_width(Length::Shrink).into()
                };

                let editor: Element<_> = if self.selected_file.is_some() {
                    self.text_editor_component()
                } else {
                    container(text("файл не выбран"))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .into()
                };

                let content = column![
                    text_input("поиск", &self.query).on_input(Message::QueryChanged),
                    editor,
                    scrollable(column(
                        self.log
                            .iter()
                            .cloned()
                            .map(|l| text(l).into())
                            .collect::<Vec<Element<Message>>>()
                    )),
                ]
                .spacing(10);

                let body = row![sidebar, content].spacing(10);

                let page = column![
                    menu,
                    mode_bar,
                    file_menu,
                    warning,
                    dirty_warning,
                    body,
                    text("Готово")
                ]
                .spacing(10);

                if self.show_delete_confirm {
                    let modal_content = container(
                        column![
                            text("Удалить выбранный файл?"),
                            row![
                                button("Да").on_press(Message::DeleteFile),
                                button("Нет").on_press(Message::CancelDeleteFile)
                            ]
                            .spacing(5)
                        ]
                        .spacing(10),
                    );
                    Modal::new(page, modal_content)
                        .on_blur(Message::CancelDeleteFile)
                        .into()
                } else {
                    page.into()
                }
            }
            Screen::VisualEditor { .. } => {
                let settings_label = if self.settings.language == Language::Russian {
                    "Настройки"
                } else {
                    "Settings"
                };
                let menu = row![
                    button("Разбор").on_press(Message::RunParse),
                    button("Поиск").on_press(Message::RunSearch),
                    button("Журнал Git").on_press(Message::RunGitLog),
                    button("Экспорт").on_press(Message::RunExport),
                    button(settings_label).on_press(Message::OpenSettings),
                ]
                .spacing(10);

                let sidebar = container(self.file_tree()).width(200);

                let rename_btn: Element<_> = if self.selected_file.is_some() {
                    button("Переименовать").on_press(Message::RenameFile).into()
                } else {
                    button("Переименовать").into()
                };
                let delete_btn: Element<_> = if self.selected_file.is_some() {
                    button("Удалить")
                        .on_press(Message::RequestDeleteFile)
                        .into()
                } else {
                    button("Удалить").into()
                };
                let save_label = if self.dirty {
                    "Сохранить*"
                } else {
                    "Сохранить"
                };
                let save_btn: Element<_> = if self.selected_file.is_some() {
                    button(save_label).on_press(Message::SaveFile).into()
                } else {
                    button(save_label).into()
                };
                let text_btn: Element<_> =
                    button("Text").on_press(Message::SwitchToTextEditor).into();
                let visual_btn: Element<_> = button("Visual").into();
                let mode_bar = row![text_btn, visual_btn, save_btn].spacing(5);

                let file_menu = row![
                    text_input("новый файл", &self.new_file_name)
                        .on_input(Message::NewFileNameChanged),
                    button("Создать файл").on_press(Message::CreateFile),
                    text_input("новый каталог", &self.new_folder_name)
                        .on_input(Message::NewFolderNameChanged),
                    button("Создать папку").on_press(Message::CreateFolder),
                    text_input("новое имя", &self.rename_file_name)
                        .on_input(Message::RenameFileNameChanged),
                    rename_btn,
                    delete_btn,
                ]
                .spacing(5);

                let warning: Element<_> = if self.show_create_file_confirm {
                    row![
                        text("Файл уже существует. Перезаписать?"),
                        button("Да").on_press(Message::ConfirmCreateFile),
                        button("Нет").on_press(Message::CancelCreateFile)
                    ]
                    .spacing(5)
                    .into()
                } else {
                    Space::with_width(Length::Shrink).into()
                };

                let dirty_warning: Element<_> = if self.pending_action.is_some() {
                    row![
                        text("Есть несохранённые изменения. Продолжить?"),
                        button("Да").on_press(Message::ConfirmDiscard),
                        button("Нет").on_press(Message::CancelDiscard)
                    ]
                    .spacing(5)
                    .into()
                } else {
                    Space::with_width(Length::Shrink).into()
                };

                let editor: Element<_> = if self.selected_file.is_some() {
                    self.visual_editor_component()
                } else {
                    container(text("файл не выбран"))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .into()
                };

                let content = column![
                    text_input("поиск", &self.query).on_input(Message::QueryChanged),
                    editor,
                    scrollable(column(
                        self.log
                            .iter()
                            .cloned()
                            .map(|l| text(l).into())
                            .collect::<Vec<Element<Message>>>()
                    )),
                ]
                .spacing(10);

                let body = row![sidebar, content].spacing(10);

                let page = column![
                    menu,
                    mode_bar,
                    file_menu,
                    warning,
                    dirty_warning,
                    body,
                    text("Готово")
                ]
                .spacing(10);

                if self.show_delete_confirm {
                    let modal_content = container(
                        column![
                            text("Удалить выбранный файл?"),
                            row![
                                button("Да").on_press(Message::DeleteFile),
                                button("Нет").on_press(Message::CancelDeleteFile)
                            ]
                            .spacing(5)
                        ]
                        .spacing(10),
                    );
                    Modal::new(page, modal_content)
                        .on_blur(Message::CancelDeleteFile)
                        .into()
                } else {
                    page.into()
                }
            }
            Screen::Settings => {
                let hotkeys = &self.settings.hotkeys;
                let create_label = if self.hotkey_capture == Some(HotkeyField::CreateFile) {
                    String::from("...")
                } else {
                    hotkeys.create_file.to_string()
                };
                let save_label = if self.hotkey_capture == Some(HotkeyField::SaveFile) {
                    String::from("...")
                } else {
                    hotkeys.save_file.to_string()
                };
                let rename_label = if self.hotkey_capture == Some(HotkeyField::RenameFile) {
                    String::from("...")
                } else {
                    hotkeys.rename_file.to_string()
                };
                let delete_label = if self.hotkey_capture == Some(HotkeyField::DeleteFile) {
                    String::from("...")
                } else {
                    hotkeys.delete_file.to_string()
                };
                let warning: Element<_> = if let Some(w) = &self.settings_warning {
                    text(w.clone()).into()
                } else {
                    Space::with_height(Length::Shrink).into()
                };
                let content = column![
                    text("Settings / Настройки"),
                    row![
                        text("Тема"),
                        pick_list(
                            &AppTheme::ALL[..],
                            Some(self.settings.theme),
                            Message::ThemeSelected
                        )
                    ]
                    .spacing(10),
                    row![
                        text("Язык"),
                        pick_list(
                            &Language::ALL[..],
                            Some(self.settings.language),
                            Message::LanguageSelected
                        )
                    ]
                    .spacing(10),
                    row![
                        text("Создать файл"),
                        button(text(create_label))
                            .on_press(Message::StartCaptureHotkey(HotkeyField::CreateFile))
                    ]
                    .spacing(10),
                    row![
                        text("Сохранить файл"),
                        button(text(save_label))
                            .on_press(Message::StartCaptureHotkey(HotkeyField::SaveFile))
                    ]
                    .spacing(10),
                    row![
                        text("Переименовать файл"),
                        button(text(rename_label))
                            .on_press(Message::StartCaptureHotkey(HotkeyField::RenameFile))
                    ]
                    .spacing(10),
                    row![
                        text("Удалить файл"),
                        button(text(delete_label))
                            .on_press(Message::StartCaptureHotkey(HotkeyField::DeleteFile))
                    ]
                    .spacing(10),
                    warning,
                    row![
                        button("Save / Сохранить").on_press(Message::SaveSettings),
                        button("Back / Назад").on_press(Message::CloseSettings)
                    ]
                    .spacing(10)
                ]
                .align_items(alignment::Alignment::Center)
                .spacing(20);

                container(content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x()
                    .center_y()
                    .into()
            }
        }
    }
}

fn pick_folder() -> impl std::future::Future<Output = Option<PathBuf>> {
    async {
        tokio::task::spawn_blocking(|| rfd::FileDialog::new().pick_folder())
            .await
            .ok()
            .flatten()
    }
}

impl MulticodeApp {
    /// Возвращает путь к корню проекта, если он выбран
    fn current_root_path(&self) -> Option<PathBuf> {
        match &self.screen {
            Screen::TextEditor { root } | Screen::VisualEditor { root } => Some(root.clone()),
            Screen::ProjectPicker => None,
            Screen::Settings => self.settings.last_folder.clone(),
        }
    }

    /// Строковое представление корневого каталога
    fn current_root(&self) -> String {
        self.current_root_path()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default()
    }
    fn load_files(&self, root: PathBuf) -> Command<Message> {
        Command::perform(
            async move {
                tokio::task::spawn_blocking(move || {
                    fn visit(dir: &Path) -> Vec<FileEntry> {
                        let mut entries = Vec::new();
                        if let Ok(read) = std::fs::read_dir(dir) {
                            let mut read: Vec<_> = read.flatten().collect();
                            read.sort_by_key(|e| e.path());
                            for entry in read {
                                if let Ok(ft) = entry.file_type() {
                                    let path = entry.path();
                                    if ft.is_dir() {
                                        let children = visit(&path);
                                        entries.push(FileEntry {
                                            path,
                                            ty: EntryType::Dir,
                                            children,
                                        });
                                    } else if ft.is_file() {
                                        entries.push(FileEntry {
                                            path,
                                            ty: EntryType::File,
                                            children: Vec::new(),
                                        });
                                    }
                                }
                            }
                        }
                        entries
                    }

                    visit(&root)
                })
                .await
                .unwrap()
            },
            Message::FilesLoaded,
        )
    }

    fn collect_files(entries: &[FileEntry], out: &mut Vec<PathBuf>) {
        for entry in entries {
            match entry.ty {
                EntryType::File => out.push(entry.path.clone()),
                EntryType::Dir => Self::collect_files(&entry.children, out),
            }
        }
    }

    fn file_paths(&self) -> Vec<PathBuf> {
        let mut out = Vec::new();
        Self::collect_files(&self.files, &mut out);
        out
    }

    fn text_editor_component(&self) -> Element<Message> {
        text_editor(&self.editor)
            .on_action(Message::FileContentEdited)
            .height(Length::Fill)
            .into()
    }

    fn visual_editor_component(&self) -> Element<Message> {
        container(text("visual editor stub"))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn view_entries(&self, entries: &[FileEntry], depth: u16) -> Element<Message> {
        let mut rows = Vec::new();
        for entry in entries {
            let indent = Space::with_width(Length::Fixed((depth * 20) as f32));
            match &entry.ty {
                EntryType::File => {
                    let name = entry
                        .path
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_string();
                    let row = row![
                        indent,
                        MouseArea::new(
                            button(text(name)).on_press(Message::SelectFile(entry.path.clone())),
                        )
                        .on_right_press(Message::ShowContextMenu(entry.path.clone())),
                    ]
                    .into();
                    rows.push(row);
                }
                EntryType::Dir => {
                    let name = entry
                        .path
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_string();
                    let expanded = self.expanded_dirs.contains(&entry.path);
                    let icon = if expanded { "▼" } else { "▶" };
                    let header = row![
                        indent,
                        MouseArea::new(
                            button(row![text(icon), text(name)])
                                .on_press(Message::ToggleDir(entry.path.clone())),
                        )
                        .on_right_press(Message::ShowContextMenu(entry.path.clone())),
                    ]
                    .into();
                    rows.push(header);
                    if expanded {
                        rows.push(self.view_entries(&entry.children, depth + 1));
                    }
                }
            }
        }
        column(rows).into()
    }

    fn file_tree(&self) -> Element<Message> {
        scrollable(self.view_entries(&self.files, 0)).into()
    }
}
