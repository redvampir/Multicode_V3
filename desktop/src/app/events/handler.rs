use super::Message;
use crate::app::io::{pick_file, pick_folder};
use crate::app::ui::ContextMenu;
use crate::app::{EditorMode, Hotkey, HotkeyField, MulticodeApp, OpenFile, PendingAction, Screen};
use chrono::Utc;
use iced::widget::text_editor::{self, Content};
use iced::{keyboard, window, Command, Event};
use multicode_core::{
    blocks, export, git,
    meta::{self, VisualMeta, DEFAULT_VERSION},
    parser::{self, Lang},
    search,
};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;
use tokio::fs;

impl MulticodeApp {
    pub fn handle_message(&mut self, message: Message) -> Command<Message> {
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
                if modifiers.control() && !modifiers.alt() && !modifiers.shift() {
                    if let keyboard::Key::Character(c) = &key {
                        match c.to_lowercase().as_str() {
                            "n" => return self.handle_message(Message::CreateFile),
                            "s" => return self.handle_message(Message::SaveFile),
                            "f" => return self.handle_message(Message::ToggleSearchPanel),
                            _ => {}
                        }
                    }
                }
                if !modifiers.control() && !modifiers.alt() && !modifiers.shift() {
                    if let keyboard::Key::Named(keyboard::key::Named::F2) = key.clone() {
                        return self.handle_message(Message::RenameFile);
                    }
                    if let keyboard::Key::Named(keyboard::key::Named::Delete) = key.clone() {
                        return self.handle_message(Message::RequestDeleteFile);
                    }
                }
                let hotkeys = &self.settings.hotkeys;
                if hotkeys.create_file.matches(&key, modifiers) {
                    return self.handle_message(Message::CreateFile);
                }
                if hotkeys.save_file.matches(&key, modifiers) {
                    return self.handle_message(Message::SaveFile);
                }
                if hotkeys.rename_file.matches(&key, modifiers) {
                    return self.handle_message(Message::RenameFile);
                }
                if hotkeys.delete_file.matches(&key, modifiers) {
                    return self.handle_message(Message::RequestDeleteFile);
                }
                Command::none()
            }
            Message::IcedEvent(Event::Window(window::Event::FileDropped(path))) => {
                if path.is_dir() {
                    return self.handle_message(Message::FolderPicked(Some(path)));
                } else if path.is_file() {
                    return self.handle_message(Message::FilePicked(Some(path)));
                }
                Command::none()
            }
            Message::IcedEvent(_) => Command::none(),
            Message::ThemeSelected(theme) => {
                self.settings.theme = theme;
                Command::none()
            }
            Message::SyntectThemeSelected(theme) => {
                self.settings.syntect_theme = theme;
                Command::none()
            }
            Message::LanguageSelected(lang) => {
                self.settings.language = lang;
                Command::none()
            }
            Message::ToggleLineNumbers(value) => {
                self.settings.show_line_numbers = value;
                Command::none()
            }
            Message::ToggleStatusBar(value) => {
                self.settings.show_status_bar = value;
                Command::none()
            }
            Message::ToggleToolbar(value) => {
                self.settings.show_toolbar = value;
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
                    return self.handle_message(Message::SaveSettings);
                }
                Command::none()
            }
            Message::SwitchToVisualEditor => {
                if let Some(root) = self.current_root_path() {
                    self.screen = Screen::VisualEditor { root: root.clone() };
                    self.settings.editor_mode = EditorMode::Visual;
                    return self.handle_message(Message::SaveSettings);
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
                        self.handle_message(Message::SaveSettings),
                    ]);
                }
                Command::none()
            }
            Message::PickFile => Command::perform(pick_file(), Message::FilePicked),
            Message::FilePicked(path) => {
                if let Some(file_path) = path {
                    if let Some(root) = file_path.parent().map(|p| p.to_path_buf()) {
                        self.settings.last_folder = Some(root.clone());
                        self.screen = match self.settings.editor_mode {
                            EditorMode::Text => Screen::TextEditor { root: root.clone() },
                            EditorMode::Visual => Screen::VisualEditor { root: root.clone() },
                        };
                        multicode_core::meta::watch::spawn(self.sender.clone());
                        return Command::batch([
                            self.load_files(root),
                            self.handle_message(Message::SaveSettings),
                            self.handle_message(Message::SelectFile(file_path)),
                        ]);
                    }
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
            Message::SearchTermChanged(s) => {
                self.search_term = s;
                Command::none()
            }
            Message::ReplaceTermChanged(s) => {
                self.replace_term = s;
                Command::none()
            }
            Message::Find => {
                self.perform_search();
                Command::none()
            }
            Message::FindNext => {
                if self.search_results.is_empty() {
                    self.perform_search();
                } else if let Some(idx) = self.current_match {
                    let len = self.search_results.len();
                    self.current_match = Some((idx + 1) % len);
                    self.focus_current_match();
                }
                Command::none()
            }
            Message::FindPrev => {
                if self.search_results.is_empty() {
                    self.perform_search();
                } else if let Some(idx) = self.current_match {
                    let len = self.search_results.len();
                    self.current_match = Some((idx + len - 1) % len);
                    self.focus_current_match();
                }
                Command::none()
            }
            Message::Replace => {
                if let Some(idx) = self.current_match {
                    if let Some((line, range)) = self.search_results.get(idx).cloned() {
                        self.move_cursor_to(line, range.start);
                        self.select_range(range.end - range.start);
                        let replacement = Arc::new(self.replace_term.clone());
                        if let Some(f) = self.current_file_mut() {
                            f.editor
                                .perform(text_editor::Action::Edit(text_editor::Edit::Paste(
                                    replacement,
                                )));
                            f.content = f.editor.text();
                            f.dirty = true;
                        }
                        self.perform_search();
                    }
                }
                Command::none()
            }
            Message::ReplaceAll => {
                if !self.search_term.is_empty() {
                    let search = self.search_term.clone();
                    let replace = self.replace_term.clone();
                    if let Some(f) = self.current_file_mut() {
                        f.content = f.editor.text().replace(&search, &replace);
                        f.editor = Content::with_text(&f.content);
                        f.dirty = true;
                    }
                    self.perform_search();
                }
                Command::none()
            }
            Message::ToggleSearchPanel => {
                self.show_search_panel = !self.show_search_panel;
                Command::none()
            }
            Message::AutoComplete => {
                if let Some(f) = self.current_file_mut() {
                    if let Some(lang) = detect_lang(&f.path) {
                        if let Some(tree) = parser::parse(&f.content, lang, None) {
                            let blocks = parser::parse_to_blocks(&tree);
                            if let Some(s) = blocks
                                .iter()
                                .find(|b| b.kind.starts_with("Function/Define"))
                            {
                                f.content.push_str(&format!("\n{}", s.kind));
                                f.editor = Content::with_text(&f.content);
                                f.dirty = true;
                            }
                        }
                    }
                }
                Command::none()
            }
            Message::AutoFormat => {
                if let Some(f) = self.current_file_mut() {
                    if let Some(lang) = detect_lang(&f.path) {
                        if parser::parse(&f.content, lang, None).is_some() {
                            let formatted = f
                                .content
                                .lines()
                                .map(|l| l.trim())
                                .collect::<Vec<_>>()
                                .join("\n");
                            f.content = formatted.clone();
                            f.editor = Content::with_text(&f.content);
                            f.dirty = true;
                        }
                    }
                }
                Command::none()
            }
            Message::SelectFile(path) => {
                self.context_menu = None;
                if let Some(idx) = self.open_files.iter().position(|f| f.path == path) {
                    self.active_file = Some(idx);
                    self.search_results.clear();
                    self.current_match = None;
                    Command::none()
                } else {
                    self.search_results.clear();
                    self.current_match = None;
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
            }
            Message::FileLoaded(Ok((path, content))) => {
                let editor = Content::with_text(&content);
                self.open_files.push(OpenFile {
                    path,
                    content,
                    editor,
                    dirty: false,
                });
                self.active_file = Some(self.open_files.len() - 1);
                self.rename_file_name.clear();
                self.search_results.clear();
                self.current_match = None;
                Command::none()
            }
            Message::FileLoaded(Err(e)) => {
                self.log.push(format!("ошибка чтения: {e}"));
                Command::none()
            }
            Message::FileContentEdited(action) => {
                if let Some(f) = self.current_file_mut() {
                    let is_edit = action.is_edit();
                    f.editor.perform(action);
                    f.content = f.editor.text();
                    if is_edit {
                        f.dirty = true;
                    }
                }
                Command::none()
            }
            Message::SaveFile => {
                if let Some(f) = self.current_file_mut() {
                    let path = f.path.clone();
                    let meta = VisualMeta {
                        version: DEFAULT_VERSION,
                        id: "root".into(),
                        x: 0.0,
                        y: 0.0,
                        tags: Vec::new(),
                        links: Vec::new(),
                        anchors: Vec::new(),
                        tests: Vec::new(),
                        extends: None,
                        origin: None,
                        translations: HashMap::new(),
                        ai: None,
                        extras: None,
                        updated_at: Utc::now(),
                    };
                    let content = meta::upsert(&f.content, &meta);
                    f.content = content.clone();
                    f.editor = Content::with_text(&f.content);
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
                self.set_dirty(false);
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
            Message::NewDirectoryNameChanged(s) => {
                self.new_directory_name = s;
                Command::none()
            }
            Message::CreateTargetChanged(t) => {
                self.create_target = t;
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
                self.open_files.push(OpenFile {
                    path: path.clone(),
                    content: String::new(),
                    editor: Content::new(),
                    dirty: false,
                });
                self.active_file = Some(self.open_files.len() - 1);
                return self.load_files(self.current_root_path().unwrap());
            }
            Message::FileCreated(Err(e)) => {
                self.log.push(format!("ошибка создания: {e}"));
                Command::none()
            }
            Message::CreateDirectory => {
                if let Some(root) = self.current_root_path() {
                    let name = self.new_directory_name.clone();
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
                        Message::DirectoryCreated,
                    );
                }
                Command::none()
            }
            Message::DirectoryCreated(Ok(path)) => {
                self.log.push(format!("создан каталог {}", path.display()));
                self.new_directory_name.clear();
                return self.load_files(self.current_root_path().unwrap());
            }
            Message::DirectoryCreated(Err(e)) => {
                self.log.push(format!("ошибка создания каталога: {e}"));
                Command::none()
            }
            Message::RenameFileNameChanged(s) => {
                self.rename_file_name = s;
                Command::none()
            }
            Message::RenameFile => {
                self.context_menu = None;
                if let Some(old_path) = self.current_file().map(|f| f.path.clone()) {
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
                if let Some(i) = self.active_file {
                    self.open_files[i].path = path.clone();
                }
                self.rename_file_name.clear();
                return self.load_files(self.current_root_path().unwrap());
            }
            Message::FileRenamed(Err(e)) => {
                self.log.push(format!("ошибка переименования: {e}"));
                Command::none()
            }
            Message::RequestDeleteFile => {
                if self.active_file.is_some() {
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
                if let Some(path) = self.current_file().map(|f| f.path.clone()) {
                    if self.is_dirty() {
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
                if let Some(idx) = self.open_files.iter().position(|f| f.path == path) {
                    self.open_files.remove(idx);
                    if let Some(active) = self.active_file {
                        if active >= idx {
                            if self.open_files.is_empty() {
                                self.active_file = None;
                            } else {
                                self.active_file = Some(active.saturating_sub(1));
                            }
                        }
                    }
                }
                return self.load_files(self.current_root_path().unwrap());
            }
            Message::FileDeleted(Err(e)) => {
                self.log.push(format!("ошибка удаления: {e}"));
                Command::none()
            }
            Message::CloseFile(idx) => {
                if let Some(file) = self.open_files.get(idx) {
                    if file.dirty {
                        let path = file.path.clone();
                        let content = file.content.clone();
                        return Command::perform(
                            async move {
                                fs::write(&path, content)
                                    .await
                                    .map(|_| idx)
                                    .map_err(|e| format!("{}", e))
                            },
                            Message::FileClosed,
                        );
                    }
                }
                if idx < self.open_files.len() {
                    self.open_files.remove(idx);
                    if let Some(active) = self.active_file {
                        if active >= idx {
                            if self.open_files.is_empty() {
                                self.active_file = None;
                            } else {
                                self.active_file = Some(active.saturating_sub(1));
                            }
                        }
                    }
                }
                Command::none()
            }
            Message::FileClosed(Ok(idx)) => {
                if idx < self.open_files.len() {
                    self.open_files.remove(idx);
                    if let Some(active) = self.active_file {
                        if active >= idx {
                            if self.open_files.is_empty() {
                                self.active_file = None;
                            } else {
                                self.active_file = Some(active.saturating_sub(1));
                            }
                        }
                    }
                }
                Command::none()
            }
            Message::FileClosed(Err(e)) => {
                self.log.push(format!("ошибка сохранения: {e}"));
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
                self.context_menu = Some(ContextMenu::new(path));
                Command::none()
            }
            Message::CloseContextMenu => {
                self.context_menu = None;
                Command::none()
            }
            Message::ConfirmDiscard => {
                self.set_dirty(false);
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
}

impl MulticodeApp {
    fn perform_search(&mut self) {
        self.search_results.clear();
        let term = self.search_term.clone();
        if !term.is_empty() {
            let lines: Vec<String> = if let Some(f) = self.current_file() {
                f.editor.lines().map(|l| l.to_string()).collect()
            } else {
                Vec::new()
            };
            for (i, line) in lines.iter().enumerate() {
                let mut start = 0;
                while let Some(pos) = line[start..].find(&term) {
                    let s = start + pos;
                    let e = s + term.len();
                    self.search_results.push((i, s..e));
                    start = e;
                }
            }
        }
        self.current_match = if self.search_results.is_empty() {
            None
        } else {
            Some(0)
        };
        self.focus_current_match();
    }

    fn focus_current_match(&mut self) {
        if let Some(idx) = self.current_match {
            if let Some((line, range)) = self.search_results.get(idx).cloned() {
                self.move_cursor_to(line, range.start);
                self.select_range(range.end - range.start);
            }
        }
    }

    fn move_cursor_to(&mut self, line: usize, column: usize) {
        if let Some(f) = self.current_file_mut() {
            f.editor.perform(text_editor::Action::Move(
                text_editor::Motion::DocumentStart,
            ));
            for _ in 0..line {
                f.editor
                    .perform(text_editor::Action::Move(text_editor::Motion::Down));
            }
            f.editor
                .perform(text_editor::Action::Move(text_editor::Motion::Home));
            for _ in 0..column {
                f.editor
                    .perform(text_editor::Action::Move(text_editor::Motion::Right));
            }
        }
    }

    fn select_range(&mut self, len: usize) {
        if let Some(f) = self.current_file_mut() {
            for _ in 0..len {
                f.editor
                    .perform(text_editor::Action::Select(text_editor::Motion::Right));
            }
        }
    }
}

fn detect_lang(path: &Path) -> Option<Lang> {
    match path.extension().and_then(|e| e.to_str())? {
        "rs" => Some(Lang::Rust),
        "py" => Some(Lang::Python),
        "js" => Some(Lang::JavaScript),
        "ts" => Some(Lang::TypeScript),
        "css" => Some(Lang::Css),
        "html" => Some(Lang::Html),
        "go" => Some(Lang::Go),
        _ => None,
    }
}
