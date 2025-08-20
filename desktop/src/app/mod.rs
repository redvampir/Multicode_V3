pub mod command_palette;
pub mod diff;
pub mod events;
pub mod io;
pub mod ui;

use crate::modal::Modal;
use diff::DiffView;
use events::Message;
use iced::futures::stream;
use iced::widget::{
    button, checkbox, column, container, pick_list, row, scrollable, spinner, text, text_editor,
    text_input,
    tooltip::{self, Tooltip},
    svg::{Handle, Svg},
    Space,
};
use iced::{
    alignment, event, keyboard, subscription, theme, Application, Command, Element, Length,
    Settings, Subscription, Theme,
};
use tokio::{fs, process::Child, sync::broadcast};
use ui::{ContextMenu, THEME_SET};

const TERMINAL_HELP: &str = include_str!("../../assets/terminal-help.md");

const CREATE_ICON: &[u8] = include_bytes!("../../assets/create.svg");
const RENAME_ICON: &[u8] = include_bytes!("../../assets/rename.svg");
const DELETE_ICON: &[u8] = include_bytes!("../../assets/delete.svg");

use directories::ProjectDirs;
use multicode_core::{git, meta::VisualMeta, BlockInfo};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::ops::Range;
use std::path::PathBuf;

pub fn run(path: Option<PathBuf>) -> iced::Result {
    let settings = UserSettings::load();
    let flags = path.or_else(|| settings.last_folders.first().cloned());
    MulticodeApp::run(Settings {
        flags,
        ..Settings::default()
    })
}

#[derive(Debug)]
pub struct MulticodeApp {
    screen: Screen,
    files: Vec<FileEntry>,
    tabs: Vec<Tab>,
    /// индекс активной вкладки
    active_tab: Option<usize>,
    /// строка поиска
    search_term: String,
    /// строка замены
    replace_term: String,
    /// найденные совпадения
    search_results: Vec<(usize, Range<usize>)>,
    /// отображать панель поиска
    show_search_panel: bool,
    /// текущий индекс совпадения
    current_match: Option<usize>,
    /// имя для создания нового файла
    new_file_name: String,
    /// имя для создания новой директории
    new_directory_name: String,
    /// что создавать: файл или директорию
    create_target: CreateTarget,
    /// новое имя при переименовании
    rename_file_name: String,
    query: String,
    show_command_palette: bool,
    log: Vec<String>,
    /// результаты поиска по проекту
    project_search_results: Vec<(PathBuf, usize, String)>,
    /// строка для перехода после открытия файла
    goto_line: Option<usize>,
    show_terminal: bool,
    terminal_cmd: String,
    terminal_child: Option<Child>,
    show_terminal_help: bool,
    sender: broadcast::Sender<String>,
    settings: UserSettings,
    expanded_dirs: HashSet<PathBuf>,
    context_menu: Option<ContextMenu>,
    /// отображать подтверждение перезаписи файла
    show_create_file_confirm: bool,
    /// отображать подтверждение удаления файла
    show_delete_confirm: bool,
    /// ожидаемое действие при подтверждении потери изменений
    pending_action: Option<PendingAction>,
    hotkey_capture: Option<HotkeyField>,
    shortcut_capture: Option<String>,
    settings_warning: Option<String>,
    loading: bool,
    diff_error: Option<String>,
    show_meta_dialog: bool,
    meta_tags: String,
    meta_links: String,
    meta_comment: String,
    show_meta_panel: bool,
}

#[derive(Debug, Clone)]
pub enum Screen {
    ProjectPicker,
    TextEditor { root: PathBuf },
    VisualEditor { root: PathBuf },
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
    path: PathBuf,
    ty: EntryType,
    children: Vec<FileEntry>,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub line: usize,
    pub range: Range<usize>,
    pub message: String,
}

#[derive(Debug)]
pub struct Tab {
    path: PathBuf,
    content: String,
    editor: text_editor::Content,
    dirty: bool,
    blame: HashMap<usize, git::BlameLine>,
    diagnostics: Vec<Diagnostic>,
    blocks: Vec<BlockInfo>,
    meta: Option<VisualMeta>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreateTarget {
    File,
    Directory,
}

impl CreateTarget {
    const ALL: [CreateTarget; 2] = [CreateTarget::File, CreateTarget::Directory];
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
#[serde(default)]
pub struct Hotkeys {
    create_file: Hotkey,
    save_file: Hotkey,
    rename_file: Hotkey,
    delete_file: Hotkey,
    next_diff: Hotkey,
    prev_diff: Hotkey,
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
pub enum Language {
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

fn default_syntect_theme() -> String {
    "InspiredGitHub".into()
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserSettings {
    #[serde(default)]
    last_folders: Vec<PathBuf>,
    #[serde(default)]
    default_entry: Option<PathBuf>,
    #[serde(default)]
    hotkeys: Hotkeys,
    #[serde(default)]
    shortcuts: HashMap<String, Hotkey>,
    #[serde(default)]
    editor_mode: EditorMode,
    #[serde(default)]
    theme: AppTheme,
    #[serde(default = "default_syntect_theme")]
    syntect_theme: String,
    #[serde(default)]
    language: Language,
    #[serde(default)]
    show_line_numbers: bool,
    #[serde(default)]
    show_status_bar: bool,
    #[serde(default = "default_true")]
    show_toolbar: bool,
    #[serde(default)]
    show_markdown_preview: bool,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            last_folders: Vec::new(),
            default_entry: None,
            hotkeys: Hotkeys::default(),
            shortcuts: HashMap::new(),
            editor_mode: EditorMode::Text,
            theme: AppTheme::default(),
            syntect_theme: default_syntect_theme(),
            language: Language::default(),
            show_line_numbers: true,
            show_status_bar: true,
            show_toolbar: true,
            show_markdown_preview: false,
        }
    }
}

impl UserSettings {
    const MAX_RECENT: usize = 5;

    fn add_recent_folder(&mut self, path: PathBuf) {
        self.last_folders.retain(|p| p != &path);
        self.last_folders.insert(0, path);
        if self.last_folders.len() > Self::MAX_RECENT {
            self.last_folders.truncate(Self::MAX_RECENT);
        }
    }

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
    type Flags = Option<PathBuf>;

    fn new(flags: Option<PathBuf>) -> (Self, Command<Message>) {
        let mut settings = UserSettings::load();
        if let Some(path) = flags {
            settings.add_recent_folder(path);
        }
        let (sender, _) = broadcast::channel(100);

        let mut app = MulticodeApp {
            screen: if let Some(path) = settings.last_folders.first().cloned() {
                match settings.editor_mode {
                    EditorMode::Text => Screen::TextEditor { root: path },
                    EditorMode::Visual => Screen::VisualEditor { root: path },
                }
            } else {
                Screen::ProjectPicker
            },
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
            query: String::new(),
            show_command_palette: false,
            log: Vec::new(),
            project_search_results: Vec::new(),
            goto_line: None,
            show_terminal: false,
            terminal_cmd: String::new(),
            terminal_child: None,
            show_terminal_help: false,
            sender,
            settings,
            expanded_dirs: HashSet::new(),
            context_menu: None,
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
            show_meta_panel: false,
        };

        let cmd = match &app.screen {
            Screen::TextEditor { root } | Screen::VisualEditor { root } => {
                let mut cmds = vec![app.load_files(root.clone())];
                if let Some(entry) = app.settings.default_entry.clone() {
                    cmds.push(app.handle_message(Message::SelectFile(entry)));
                }
                Command::batch(cmds)
            }
            _ => Command::none(),
        };

        (app, cmd)
    }

    fn title(&self) -> String {
        String::from("Multicode Desktop")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        self.handle_message(message)
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
        let (tabs, content): (Option<Element<_>>, Element<_>) = match &self.screen {
            Screen::ProjectPicker => {
                let settings_label = if self.settings.language == Language::Russian {
                    "Настройки"
                } else {
                    "Settings"
                };
                let mut content = column![
                    text("Выберите папку проекта"),
                    button("Выбрать").on_press(Message::PickFolder),
                    button("Выбрать файл").on_press(Message::PickFile),
                    button(settings_label).on_press(Message::OpenSettings),
                ]
                .align_items(alignment::Alignment::Center)
                .spacing(20);

                if !self.settings.last_folders.is_empty() {
                    let open_label = if self.settings.language == Language::Russian {
                        "Открыть"
                    } else {
                        "Open"
                    };
                    content = content.push(text("Недавние проекты:"));
                    for path in &self.settings.last_folders {
                        let path_str = path.to_string_lossy().to_string();
                        content = content.push(
                            row![
                                text(path_str).width(Length::FillPortion(1)),
                                button(open_label).on_press(Message::OpenRecent(path.clone())),
                            ]
                            .spacing(10),
                        );
                    }
                }

                let picker = container(content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x()
                    .center_y();
                let content = column![picker, self.status_bar_component()].spacing(10);
                let content = row![self.sidebar(), content].spacing(10).into();
                (None, content)
            }
            Screen::TextEditor { .. } => {
                let sidebar = self.sidebar();

                let tabs = row(self
                    .tabs
                    .iter()
                    .enumerate()
                    .map(|(i, f)| {
                        let name = f.path.file_name().unwrap().to_string_lossy().to_string();
                        row![
                            button(text(name)).on_press(Message::SelectFile(f.path.clone())),
                            button(text("x")).on_press(Message::CloseFile(i))
                        ]
                        .spacing(5)
                        .into()
                    })
                    .collect::<Vec<Element<Message>>>())
                .spacing(5);

                let file_menu = self.file_menu();

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

                let editor: Element<_> = self.text_editor_component();

                let search_panel = self.search_panel_component();

                let content = column![
                    search_panel,
                    editor,
                    self.project_search_component(),
                    self.lint_panel_component(),
                    self.terminal_component(),
                ]
                .spacing(10);

                let body = row![sidebar, content].spacing(10);

                let page = column![
                    file_menu,
                    warning,
                    dirty_warning,
                    body,
                    self.status_bar_component()
                ]
                .spacing(10);

                let mut content: Element<_> = page.into();
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
                    content = Modal::new(content, modal_content)
                        .on_blur(Message::CancelDeleteFile)
                        .into();
                }
                if self.show_terminal_help {
                    let help = container(scrollable(text(TERMINAL_HELP)))
                        .width(Length::Fixed(400.0))
                        .padding(10);
                    content = Modal::new(content, help)
                        .on_blur(Message::ShowTerminalHelp)
                        .into();
                }
                if self.show_meta_dialog {
                    let modal_content = container(
                        column![
                            text_input("Теги", &self.meta_tags).on_input(Message::MetaTagsChanged),
                            text_input("Связи", &self.meta_links)
                                .on_input(Message::MetaLinksChanged),
                            text_input("Комментарий", &self.meta_comment)
                                .on_input(Message::MetaCommentChanged),
                            row![
                                button("Сохранить").on_press(Message::SaveMeta),
                                button("Отмена").on_press(Message::CloseMetaDialog)
                            ]
                            .spacing(5),
                        ]
                        .spacing(5),
                    )
                    .width(Length::Fixed(400.0))
                    .padding(10);
                    content = Modal::new(content, modal_content)
                        .on_blur(Message::CloseMetaDialog)
                        .into();
                }
                (Some(tabs), content)
            }
            Screen::VisualEditor { .. } => {
                let sidebar = self.sidebar();

                let tabs = self.tabs_component();

                let file_menu = self.file_menu();

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

                let editor: Element<_> = self.visual_editor_component();

                let content = column![
                    text_input("поиск", &self.query).on_input(Message::QueryChanged),
                    editor,
                    self.project_search_component(),
                    self.terminal_component(),
                ]
                .spacing(10);

                let body = row![sidebar, content].spacing(10);

                let page = column![
                    file_menu,
                    warning,
                    dirty_warning,
                    body,
                    self.status_bar_component()
                ]
                .spacing(10);

                let mut content: Element<_> = page.into();
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
                    content = Modal::new(content, modal_content)
                        .on_blur(Message::CancelDeleteFile)
                        .into();
                }
                if self.show_terminal_help {
                    let help = container(scrollable(text(TERMINAL_HELP)))
                        .width(Length::Fixed(400.0))
                        .padding(10);
                    content = Modal::new(content, help)
                        .on_blur(Message::ShowTerminalHelp)
                        .into();
                }
                if self.show_meta_dialog {
                    let modal_content = container(
                        column![
                            text_input("Теги", &self.meta_tags).on_input(Message::MetaTagsChanged),
                            text_input("Связи", &self.meta_links)
                                .on_input(Message::MetaLinksChanged),
                            text_input("Комментарий", &self.meta_comment)
                                .on_input(Message::MetaCommentChanged),
                            row![
                                button("Сохранить").on_press(Message::SaveMeta),
                                button("Отмена").on_press(Message::CloseMetaDialog)
                            ]
                            .spacing(5)
                        ]
                        .spacing(5),
                    )
                    .width(Length::Fixed(400.0))
                    .padding(10);
                    content = Modal::new(content, modal_content)
                        .on_blur(Message::CloseMetaDialog)
                        .into();
                }
                (Some(tabs), content)
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
                let next_diff_label = if self.hotkey_capture == Some(HotkeyField::NextDiff) {
                    String::from("...")
                } else {
                    hotkeys.next_diff.to_string()
                };
                let prev_diff_label = if self.hotkey_capture == Some(HotkeyField::PrevDiff) {
                    String::from("...")
                } else {
                    hotkeys.prev_diff.to_string()
                };
                let syntect_themes: Vec<String> = THEME_SET.themes.keys().cloned().collect();
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
                        text("Тема подсветки"),
                        pick_list(
                            syntect_themes.clone(),
                            Some(self.settings.syntect_theme.clone()),
                            Message::SyntectThemeSelected
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
                        text("Номера строк"),
                        checkbox("", self.settings.show_line_numbers)
                            .on_toggle(Message::ToggleLineNumbers)
                    ]
                    .spacing(10),
                    row![
                        text("Статус-бар"),
                        checkbox("", self.settings.show_status_bar)
                            .on_toggle(Message::ToggleStatusBar),
                    ]
                    .spacing(10),
                    row![
                        text("Панель инструментов"),
                        checkbox("", self.settings.show_toolbar).on_toggle(Message::ToggleToolbar),
                    ]
                    .spacing(10),
                    row![
                        text("Предпросмотр Markdown"),
                        checkbox("", self.settings.show_markdown_preview)
                            .on_toggle(Message::ToggleMarkdownPreview),
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
                    row![
                        text("Следующее отличие"),
                        button(text(next_diff_label))
                            .on_press(Message::StartCaptureHotkey(HotkeyField::NextDiff))
                    ]
                    .spacing(10),
                    row![
                        text("Предыдущее отличие"),
                        button(text(prev_diff_label))
                            .on_press(Message::StartCaptureHotkey(HotkeyField::PrevDiff))
                    ]
                    .spacing(10),
                    self.shortcuts_settings_component(),
                    warning,
                    row![
                        button("Save / Сохранить").on_press(Message::SaveSettings),
                        button("Back / Назад").on_press(Message::CloseSettings)
                    ]
                    .spacing(10)
                ]
                .align_items(alignment::Alignment::Center)
                .spacing(20);

                let settings_page = container(content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x()
                    .center_y();
                let content = column![settings_page, self.status_bar_component()].spacing(10);
                let content = row![self.sidebar(), content].spacing(10).into();
                (None, content)
            }
            Screen::Diff(diff) => {
                let diff_view = container(self.diff_component(diff))
                    .width(Length::Fill)
                    .height(Length::Fill);
                let content = column![diff_view, self.status_bar_component()].spacing(10);
                let content = row![self.sidebar(), content].spacing(10).into();
                (None, content)
            }
        };
        let mut page = column![self.main_menu()];
        if let Some(tabs) = tabs {
            page = page.push(tabs);
        }
        let page: Element<_> = page
            .push(self.mode_bar())
            .push(self.toolbar())
            .push(content)
            .spacing(10)
            .into();
        let content = if self.loading {
            container(spinner())
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .into()
        } else {
            page
        };
        let content = self.command_palette_modal(content);
        self.error_modal(content)
    }
}

impl MulticodeApp {
    fn main_menu(&self) -> Element<Message> {
        let settings_label = if self.settings.language == Language::Russian {
            "Настройки"
        } else {
            "Settings"
        };
        let open_other_label = if self.settings.language == Language::Russian {
            "Открыть другой проект"
        } else {
            "Open another project"
        };
        row![
            button("Разбор").on_press(Message::RunParse),
            button("Поиск").on_press(Message::ProjectSearch(self.query.clone())),
            button("Журнал Git").on_press(Message::RunGitLog),
            button("Экспорт").on_press(Message::RunExport),
            button(open_other_label).on_press(Message::OpenProjectPicker),
            button("Терминал").on_press(Message::ToggleTerminal),
            button(settings_label).on_press(Message::OpenSettings),
        ]
        .spacing(10)
        .into()
    }

    fn mode_bar(&self) -> Element<Message> {
        let text_btn: Element<_> = if self.is_visual_mode() {
            button("Text").on_press(Message::SwitchToTextEditor).into()
        } else if matches!(self.screen, Screen::TextEditor { .. }) {
            button("Text").style(theme::Button::Primary).into()
        } else {
            button("Text").into()
        };
        let visual_btn: Element<_> = if self.is_visual_mode() {
            button("Visual").style(theme::Button::Primary).into()
        } else if matches!(self.screen, Screen::TextEditor { .. }) {
            button("Visual")
                .on_press(Message::SwitchToVisualEditor)
                .into()
        } else {
            button("Visual").into()
        };

        let save_label = if self.is_dirty() {
            "Сохранить*"
        } else {
            "Сохранить"
        };
        let save_btn: Element<_> = if self.active_tab.is_some() {
            button(save_label).on_press(Message::SaveFile).into()
        } else {
            button(save_label).into()
        };

        if matches!(&self.screen, Screen::TextEditor { .. }) {
            let autocomplete_btn: Element<_> = if self.active_tab.is_some() {
                button("Автодополнить")
                    .on_press(Message::AutoComplete)
                    .into()
            } else {
                button("Автодополнить").into()
            };
            let format_btn: Element<_> = if self.active_tab.is_some() {
                button("Форматировать").on_press(Message::AutoFormat).into()
            } else {
                button("Форматировать").into()
            };
            row![text_btn, visual_btn, save_btn, autocomplete_btn, format_btn]
                .spacing(5)
                .into()
        } else {
            row![text_btn, visual_btn, save_btn].spacing(5).into()
        }
    }

    fn file_menu(&self) -> Element<Message> {
        let create_icon = Svg::new(Handle::from_memory(CREATE_ICON))
            .width(Length::Fixed(16.0))
            .height(Length::Fixed(16.0));
        let rename_icon = Svg::new(Handle::from_memory(RENAME_ICON))
            .width(Length::Fixed(16.0))
            .height(Length::Fixed(16.0));
        let delete_icon = Svg::new(Handle::from_memory(DELETE_ICON))
            .width(Length::Fixed(16.0))
            .height(Length::Fixed(16.0));

        let create_select = pick_list(
            &CreateTarget::ALL[..],
            Some(self.create_target),
            Message::CreateTargetChanged,
        );
        let (placeholder, value, on_input_msg, create_msg): (
            &str,
            &String,
            fn(String) -> Message,
            Message,
        ) = match self.create_target {
            CreateTarget::File => (
                "новый файл",
                &self.new_file_name,
                Message::NewFileNameChanged as fn(String) -> Message,
                Message::CreateFile,
            ),
            CreateTarget::Directory => (
                "новый каталог",
                &self.new_directory_name,
                Message::NewDirectoryNameChanged as fn(String) -> Message,
                Message::CreateDirectory,
            ),
        };
        let create_input = text_input(placeholder, value).on_input(on_input_msg);
        let create_button: Element<_> = Tooltip::new(
            button(create_icon).on_press(create_msg),
            "Создать",
            tooltip::Position::Top,
        )
        .into();

        let rename_btn: Element<_> = {
            let btn = button(rename_icon);
            let btn = if self.active_tab.is_some() {
                btn.on_press(Message::RenameFile)
            } else {
                btn
            };
            Tooltip::new(btn, "Переименовать", tooltip::Position::Top).into()
        };

        let delete_btn: Element<_> = {
            let btn = button(delete_icon);
            let btn = if self.active_tab.is_some() {
                btn.on_press(Message::RequestDeleteFile)
            } else {
                btn
            };
            Tooltip::new(btn, "Удалить", tooltip::Position::Top).into()
        };

        row![
            create_select,
            create_input,
            create_button,
            text_input("новое имя", &self.rename_file_name)
                .on_input(Message::RenameFileNameChanged),
            rename_btn,
            delete_btn,
        ]
        .spacing(5)
        .into()
    }

    fn sidebar(&self) -> Element<Message> {
        container(self.file_tree()).width(200).into()
    }

    fn current_file(&self) -> Option<&Tab> {
        self.active_tab.and_then(|i| self.tabs.get(i))
    }

    fn current_file_mut(&mut self) -> Option<&mut Tab> {
        if let Some(i) = self.active_tab {
            self.tabs.get_mut(i)
        } else {
            None
        }
    }

    fn is_dirty(&self) -> bool {
        self.current_file().map(|f| f.dirty).unwrap_or(false)
    }

    fn set_dirty(&mut self, value: bool) {
        if let Some(f) = self.current_file_mut() {
            f.dirty = value;
        }
    }

    fn is_visual_mode(&self) -> bool {
        matches!(self.screen, Screen::VisualEditor { .. })
    }
    /// Возвращает путь к корню проекта, если он выбран
    fn current_root_path(&self) -> Option<PathBuf> {
        match &self.screen {
            Screen::TextEditor { root } | Screen::VisualEditor { root } => Some(root.clone()),
            Screen::Diff(_) => self.settings.last_folders.first().cloned(),
            Screen::ProjectPicker => None,
            Screen::Settings => self.settings.last_folders.first().cloned(),
        }
    }

    /// Строковое представление корневого каталога
    fn current_root(&self) -> String {
        self.current_root_path()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default()
    }
}
