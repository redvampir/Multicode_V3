use iced::futures::stream;
#[allow(unused_imports)]
use iced::widget::overlay::menu;
use iced::widget::{
    button, column, container, row, scrollable, text, text_editor, text_input, MouseArea, Space,
};
use iced::{
    alignment, event, keyboard, subscription, Application, Command, Element, Event, Length,
    Settings, Subscription, Theme,
};
use multicode_core::{blocks, export, git, search};
use tokio::{fs, sync::broadcast};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
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
    context_menu: Option<(PathBuf, menu::State)>,
    /// отображать подтверждение перезаписи файла
    show_create_file_confirm: bool,
    /// отображать подтверждение удаления файла
    show_delete_confirm: bool,
    /// есть ли несохранённые изменения
    dirty: bool,
    /// ожидаемое действие при подтверждении потери изменений
    pending_action: Option<PendingAction>,
}

#[derive(Debug, Clone)]
enum Screen {
    ProjectPicker,
    Workspace { root: PathBuf },
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
    ToggleDir(PathBuf),
    ShowContextMenu(PathBuf),
    CloseContextMenu,
    /// подтверждение потери несохранённых изменений
    ConfirmDiscard,
    /// отмена потери несохранённых изменений
    CancelDiscard,
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

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct UserSettings {
    last_folder: Option<PathBuf>,
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
                Screen::Workspace { root: path }
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
        };

        let cmd = match &app.screen {
            Screen::Workspace { root } => app.load_files(root.clone()),
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
                if modifiers.control() {
                    match key.as_ref() {
                        keyboard::Key::Character("n") => return self.update(Message::CreateFile),
                        keyboard::Key::Character("s") => return self.update(Message::SaveFile),
                        _ => {}
                    }
                } else if modifiers.is_empty() {
                    match key.as_ref() {
                        keyboard::Key::Named(keyboard::key::Named::F2) => {
                            return self.update(Message::RenameFile)
                        }
                        keyboard::Key::Named(keyboard::key::Named::Delete) => {
                            return self.update(Message::DeleteFile)
                        }
                        _ => {}
                    }
                }
                Command::none()
            }
            Message::IcedEvent(_) => Command::none(),
            Message::PickFolder => Command::perform(pick_folder(), Message::FolderPicked),
            Message::FolderPicked(path) => {
                if let Some(root) = path {
                    self.settings.last_folder = Some(root.clone());
                    self.screen = Screen::Workspace { root: root.clone() };
                    multicode_core::meta::watch::spawn(self.sender.clone());
                    return Command::batch([
                        self.load_files(root),
                        Command::perform(self.settings.clone().save(), |_| Message::SaveSettings),
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
                self.context_menu = Some((path, menu::State::new()));
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
            Message::SaveSettings => Command::none(),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        if matches!(self.screen, Screen::Workspace { .. }) {
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

    fn view(&self) -> Element<Message> {
        match &self.screen {
            Screen::ProjectPicker => {
                let content = column![
                    text("Выберите папку проекта"),
                    button("Выбрать папку").on_press(Message::PickFolder),
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
            Screen::Workspace { .. } => {
                let menu = row![
                    button("Разбор").on_press(Message::RunParse),
                    button("Поиск").on_press(Message::RunSearch),
                    button("Журнал Git").on_press(Message::RunGitLog),
                    button("Экспорт").on_press(Message::RunExport),
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
                    save_btn,
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
                    text_editor(&self.editor)
                        .on_action(Message::FileContentEdited)
                        .height(Length::Fill)
                        .into()
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

                let delete_warning: Element<_> = if self.show_delete_confirm {
                    row![
                        text("Удалить выбранный файл?"),
                        button("Да").on_press(Message::DeleteFile),
                        button("Нет").on_press(Message::CancelDeleteFile)
                    ]
                    .spacing(5)
                    .into()
                } else {
                    Space::with_width(Length::Shrink).into()
                };

                column![
                    menu,
                    file_menu,
                    delete_warning,
                    warning,
                    dirty_warning,
                    body,
                    text("Готово")
                ]
                .spacing(10)
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
            Screen::Workspace { root } => Some(root.clone()),
            Screen::ProjectPicker => None,
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
