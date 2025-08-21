use super::Message;
use crate::app::io::{pick_file, pick_file_in_dir, pick_folder};
use crate::app::{
    command_palette::COMMANDS, diff::DiffView, Diagnostic, EditorMode, EntryType, Hotkey,
    HotkeyField, MulticodeApp, PendingAction, Screen, Tab, TabDragState, ViewMode,
};
use crate::components::file_manager::{self, ContextMenu};
use crate::editor::autocomplete::{self, AutocompleteState};
use crate::editor::meta_integration::validate_meta_json;
use crate::visual::canvas::CanvasMessage;
use crate::visual::palette::PaletteMessage;
use chrono::Utc;
use iced::widget::{
    scrollable,
    text_editor::{self, Content},
};
use iced::{keyboard, window, Command, Event};
use multicode_core::{
    blocks, export, git,
    meta::{self, VisualMeta, DEFAULT_VERSION},
    parser::{self, Lang},
    search, viz_lint, BlockInfo,
};
use serde_json::json;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as TokioCommand;
use tokio::time::{sleep, Duration};

const HISTORY_LIMIT: usize = 100;

fn push_with_limit(stack: &mut VecDeque<String>, value: String) {
    if stack.len() >= HISTORY_LIMIT {
        stack.pop_front();
    }
    stack.push_back(value);
}

impl MulticodeApp {
    pub fn handle_message(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::CanvasEvent(event) => {
                match event {
                    CanvasMessage::BlockDragged { index, position } => {
                        if let Some(tab) = self.current_file_mut() {
                            if let Some(block) = tab.blocks.get_mut(index) {
                                block.x = position.x as f64;
                                block.y = position.y as f64;
                                tab.dirty = true;
                            }
                        }
                    }
                    CanvasMessage::Dropped { position } => {
                        if let Some(mut block) = self.palette_drag.take() {
                            block.x = position.x as f64;
                            block.y = position.y as f64;
                            if let Some(tab) = self.current_file_mut() {
                                tab.blocks.push(block);
                                tab.dirty = true;
                            }
                        }
                    }
                    CanvasMessage::TogglePalette => {
                        self.show_block_palette = !self.show_block_palette;
                        if !self.show_block_palette {
                            self.palette_query.clear();
                        }
                    }
                    CanvasMessage::BlockSelected(_)
                    | CanvasMessage::Pan { .. }
                    | CanvasMessage::Zoom { .. } => {}
                }
                Command::none()
            }
            Message::PaletteEvent(ev) => {
                match ev {
                    PaletteMessage::SearchChanged(q) => {
                        self.palette_query = q;
                    }
                    PaletteMessage::StartDrag(i) => {
                        if let Some(block) = self.palette.get(i).map(|b| b.info.clone()) {
                            self.palette_drag = Some(block);
                            self.show_block_palette = false;
                        }
                    }
                    PaletteMessage::ToggleFavorite(i) => {
                        if let Some(kind) = self.palette.get(i).map(|b| b.info.kind.clone()) {
                            if let Some(pos) = self
                                .settings
                                .block_favorites
                                .iter()
                                .position(|k| k == &kind)
                            {
                                self.settings.block_favorites.remove(pos);
                            } else {
                                self.settings.block_favorites.push(kind);
                            }
                            return self.handle_message(Message::SaveSettings);
                        }
                    }
                    PaletteMessage::Close => {
                        self.show_block_palette = false;
                        self.palette_query.clear();
                    }
                }
                Command::none()
            }
            Message::IcedEvent(Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                modifiers,
                ..
            })) => {
                if self.show_block_palette {
                    if let keyboard::Key::Named(keyboard::key::Named::Escape) = key {
                        return self.handle_message(Message::PaletteEvent(PaletteMessage::Close));
                    }
                }
                if let Some(id) = self.shortcut_capture.take() {
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
                    self.settings.shortcuts.insert(id, hk);
                    return Command::none();
                }
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
                        HotkeyField::NextDiff => self.settings.hotkeys.next_diff = hk,
                        HotkeyField::PrevDiff => self.settings.hotkeys.prev_diff = hk,
                    }
                    return Command::none();
                }
                if modifiers.alt() && !modifiers.control() && !modifiers.shift() {
                    if let keyboard::Key::Character(c) = &key {
                        if c.eq_ignore_ascii_case("l") {
                            self.settings.language = self.settings.language.next();
                            return Command::none();
                        }
                    }
                }
                if let Screen::Diff(_) = self.screen {
                    let hotkeys = &self.settings.hotkeys;
                    if hotkeys.next_diff.matches(&key, modifiers) {
                        return self.handle_message(Message::NextDiff);
                    }
                    if hotkeys.prev_diff.matches(&key, modifiers) {
                        return self.handle_message(Message::PrevDiff);
                    }
                    return Command::none();
                }
                if !matches!(
                    self.screen,
                    Screen::TextEditor { .. } | Screen::VisualEditor { .. } | Screen::Split { .. }
                ) {
                    return Command::none();
                }
                if self.autocomplete.is_some() {
                    if !modifiers.control() && !modifiers.alt() {
                        let mut accept: Option<String> = None;
                        {
                            let ac = self.autocomplete.as_mut().unwrap();
                            match key {
                                keyboard::Key::Named(keyboard::key::Named::Enter)
                                | keyboard::Key::Named(keyboard::key::Named::Tab) => {
                                    if let Some(s) = ac.current() {
                                        accept = Some(s.insert.clone());
                                    }
                                }
                                keyboard::Key::Named(keyboard::key::Named::ArrowDown) => {
                                    ac.next();
                                    return Command::none();
                                }
                                keyboard::Key::Named(keyboard::key::Named::ArrowUp) => {
                                    ac.prev();
                                    return Command::none();
                                }
                                keyboard::Key::Named(keyboard::key::Named::Escape) => {
                                    self.autocomplete = None;
                                    return Command::none();
                                }
                                _ => {}
                            }
                        }
                        if let Some(text) = accept {
                            if let Some(f) = self.current_file_mut() {
                                let insert = Arc::new(text);
                                f.editor.perform(text_editor::Action::Edit(
                                    text_editor::Edit::Paste(insert),
                                ));
                                f.content = f.editor.text();
                                f.dirty = true;
                            }
                            self.autocomplete = None;
                            return Command::none();
                        }
                    }
                }
                if modifiers.control() && modifiers.shift() && !modifiers.alt() {
                    if let keyboard::Key::Character(c) = &key {
                        if c.eq_ignore_ascii_case("p") {
                            return self.handle_message(Message::ToggleCommandPalette);
                        } else if c.eq_ignore_ascii_case("v") {
                            return self.handle_message(Message::SwitchToVisualEditor);
                        } else if c.eq_ignore_ascii_case("t") {
                            return self.handle_message(Message::SwitchToTextEditor);
                        } else if c.eq_ignore_ascii_case("s") {
                            return self.handle_message(Message::SwitchViewMode(ViewMode::Split));
                        }
                    }
                }
                if modifiers.control() && !modifiers.alt() && !modifiers.shift() {
                    if let keyboard::Key::Character(c) = &key {
                        match c.to_lowercase().as_str() {
                            "n" => return self.handle_message(Message::CreateFile),
                            "s" => return self.handle_message(Message::SaveFile),
                            "f" => return self.handle_message(Message::ToggleSearchPanel),
                            "z" => return self.handle_message(Message::Undo),
                            "y" => return self.handle_message(Message::Redo),
                            _ => {}
                        }
                    }
                }
                if !modifiers.control() && !modifiers.alt() && !modifiers.shift() {
                    if let keyboard::Key::Named(named) = &key {
                        match named {
                            keyboard::key::Named::ArrowUp => {
                                return self.handle_message(Message::NavigateUp);
                            }
                            keyboard::key::Named::ArrowDown => {
                                return self.handle_message(Message::NavigateDown);
                            }
                            keyboard::key::Named::ArrowLeft => {
                                return self.handle_message(Message::NavigateBack);
                            }
                            keyboard::key::Named::ArrowRight | keyboard::key::Named::Enter => {
                                return self.handle_message(Message::NavigateInto);
                            }
                            keyboard::key::Named::F2 => {
                                return self.handle_message(Message::RenameFile);
                            }
                            keyboard::key::Named::Delete => {
                                return self.handle_message(Message::RequestDeleteFile);
                            }
                            _ => {}
                        }
                    }
                }
                for (id, hk) in &self.settings.shortcuts {
                    if hk.matches(&key, modifiers) {
                        if let Some(cmd) = COMMANDS.iter().find(|c| c.id == id.as_str()) {
                            return self.handle_message(cmd.message.clone());
                        }
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
            Message::IcedEvent(Event::Window(_, window::Event::FileDropped(path))) => {
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
            Message::FontSizeChanged(value) => {
                if let Ok(v) = value.parse() {
                    self.settings.editor.font_size = v;
                }
                Command::none()
            }
            Message::TabWidthChanged(value) => {
                if let Ok(v) = value.parse() {
                    self.settings.editor.tab_width = v;
                }
                Command::none()
            }
            Message::ToggleAutoIndent(val) => {
                self.settings.editor.auto_indent = val;
                Command::none()
            }
            Message::ToggleLineWrapping(val) => {
                self.settings.editor.line_wrapping = val;
                Command::none()
            }
            Message::ToggleHighlightCurrentLine(val) => {
                self.settings.editor.highlight_current_line = val;
                Command::none()
            }
            Message::EditorThemeSelected(theme) => {
                self.settings.editor.theme = theme;
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
            Message::ToggleMarkdownPreview(value) => {
                self.settings.show_markdown_preview = value;
                Command::none()
            }
            Message::ToggleMetaPanel => {
                self.show_meta_panel = !self.show_meta_panel;
                Command::none()
            }
            Message::ShowMetaDialog => {
                if let Some(f) = self.current_file() {
                    let tags = f
                        .meta
                        .as_ref()
                        .map(|m| m.tags.join(", "))
                        .unwrap_or_default();
                    let links = f
                        .meta
                        .as_ref()
                        .map(|m| m.links.join(", "))
                        .unwrap_or_default();
                    let comment = f
                        .meta
                        .as_ref()
                        .and_then(|m| m.extras.as_ref())
                        .and_then(|e| e.get("comment"))
                        .and_then(|c| c.as_str())
                        .unwrap_or("")
                        .to_string();
                    self.meta_tags = tags;
                    self.meta_links = links;
                    self.meta_comment = comment;
                }
                self.show_meta_dialog = true;
                Command::none()
            }
            Message::CloseMetaDialog => {
                self.show_meta_dialog = false;
                Command::none()
            }
            Message::MetaTagsChanged(s) => {
                self.meta_tags = s;
                Command::none()
            }
            Message::MetaLinksChanged(s) => {
                self.meta_links = s;
                Command::none()
            }
            Message::MetaCommentChanged(s) => {
                self.meta_comment = s;
                Command::none()
            }
            Message::SaveMeta => {
                let tags_str = self.meta_tags.clone();
                let links_str = self.meta_links.clone();
                let comment_str = self.meta_comment.clone();
                if let Some(f) = self.current_file_mut() {
                    let mut meta = f.meta.clone().unwrap_or(VisualMeta {
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
                    });
                    meta.tags = tags_str
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    meta.links = links_str
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    if comment_str.trim().is_empty() {
                        if let Some(mut extras) = meta.extras.take() {
                            if let Some(obj) = extras.as_object_mut() {
                                obj.remove("comment");
                                if obj.is_empty() {
                                    meta.extras = None;
                                } else {
                                    meta.extras = Some(extras);
                                }
                            }
                        }
                    } else {
                        let mut extras = meta.extras.take().unwrap_or_else(|| json!({}));
                        if let Some(obj) = extras.as_object_mut() {
                            obj.insert(
                                "comment".into(),
                                serde_json::Value::String(comment_str.clone()),
                            );
                        }
                        meta.extras = Some(extras);
                    }
                    meta.updated_at = Utc::now();
                    f.content = meta::upsert(&f.content, &meta);
                    f.editor = Content::with_text(&f.content);
                    f.meta = Some(meta);
                    f.dirty = true;
                }
                self.show_meta_dialog = false;
                Command::none()
            }
            Message::StartCaptureHotkey(field) => {
                self.hotkey_capture = Some(field);
                Command::none()
            }
            Message::StartCaptureShortcut(id) => {
                self.shortcut_capture = Some(id);
                Command::none()
            }
            Message::OpenSettings => {
                self.screen = Screen::Settings;
                Command::none()
            }
            Message::CloseSettings => {
                if let Some(root) = self.settings.last_folders.first().cloned() {
                    self.screen = match self.settings.editor_mode {
                        EditorMode::Text => Screen::TextEditor { root },
                        EditorMode::Visual => Screen::VisualEditor { root },
                        EditorMode::Split => Screen::Split { root },
                    };
                } else {
                    self.screen = Screen::ProjectPicker;
                }
                Command::none()
            }
            Message::OpenProjectPicker => {
                self.screen = Screen::ProjectPicker;
                Command::none()
            }
            Message::SwitchViewMode(mode) => {
                self.view_mode = mode;
                match mode {
                    ViewMode::Code => self.handle_message(Message::SwitchToTextEditor),
                    ViewMode::Schema => self.handle_message(Message::SwitchToVisualEditor),
                    ViewMode::Split => {
                        if let Some(root) = self.current_root_path() {
                            self.screen = Screen::Split { root: root.clone() };
                            self.settings.editor_mode = EditorMode::Split;
                            return self.handle_message(Message::SaveSettings);
                        }
                        Command::none()
                    }
                }
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
                    self.settings.add_recent_folder(root.clone());
                    self.screen = match self.settings.editor_mode {
                        EditorMode::Text => Screen::TextEditor { root: root.clone() },
                        EditorMode::Visual => Screen::VisualEditor { root: root.clone() },
                        EditorMode::Split => Screen::Split { root: root.clone() },
                    };
                    multicode_core::meta::watch::spawn(self.sender.clone());
                    if let Some(entry) = self.settings.default_entry.clone() {
                        return Command::batch([
                            self.load_files(root),
                            self.handle_message(Message::SaveSettings),
                            self.handle_message(Message::SelectFile(entry)),
                        ]);
                    } else {
                        return Command::batch([
                            self.load_files(root.clone()),
                            Command::perform(pick_file_in_dir(root), Message::DefaultEntryPicked),
                            self.handle_message(Message::SaveSettings),
                        ]);
                    }
                }
                Command::none()
            }
            Message::PickFile => Command::perform(pick_file(), Message::FilePicked),
            Message::FilePicked(path) => {
                if let Some(file_path) = path {
                    if let Some(root) = file_path.parent().map(|p| p.to_path_buf()) {
                        self.settings.add_recent_folder(root.clone());
                        self.screen = match self.settings.editor_mode {
                            EditorMode::Text => Screen::TextEditor { root: root.clone() },
                            EditorMode::Visual => Screen::VisualEditor { root: root.clone() },
                            EditorMode::Split => Screen::Split { root: root.clone() },
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
            Message::OpenRecent(root) => {
                self.settings.add_recent_folder(root.clone());
                self.screen = match self.settings.editor_mode {
                    EditorMode::Text => Screen::TextEditor { root: root.clone() },
                    EditorMode::Visual => Screen::VisualEditor { root: root.clone() },
                    EditorMode::Split => Screen::Split { root: root.clone() },
                };
                multicode_core::meta::watch::spawn(self.sender.clone());
                if let Some(entry) = self.settings.default_entry.clone() {
                    Command::batch([
                        self.load_files(root),
                        self.handle_message(Message::SaveSettings),
                        self.handle_message(Message::SelectFile(entry)),
                    ])
                } else {
                    Command::batch([
                        self.load_files(root.clone()),
                        Command::perform(pick_file_in_dir(root), Message::DefaultEntryPicked),
                        self.handle_message(Message::SaveSettings),
                    ])
                }
            }
            Message::FilesLoaded(list) => {
                self.files = list;
                self.selected_path = self.files.first().map(|e| e.path.clone());
                Command::none()
            }
            Message::FileError(e) => {
                self.log.push(format!("ошибка файла: {e}"));
                Command::none()
            }
            Message::DefaultEntryPicked(path) => {
                if let Some(p) = path {
                    self.settings.default_entry = Some(p.clone());
                    return Command::batch([
                        self.handle_message(Message::SelectFile(p)),
                        self.handle_message(Message::SaveSettings),
                    ]);
                }
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
                self.autocomplete = None;
                if let Some((_line, column, line_text, content)) = self.current_file().map(|f| {
                    let (l, c) = f.editor.cursor_position();
                    let lt = f
                        .editor
                        .lines()
                        .nth(l)
                        .map(|line| line.to_string())
                        .unwrap_or_default();
                    (l, c, lt, f.editor.text())
                }) {
                    let prefix = line_text[..column]
                        .chars()
                        .rev()
                        .take_while(|ch| ch.is_alphanumeric() || *ch == '_')
                        .collect::<Vec<_>>();
                    let prefix: String = prefix.into_iter().rev().collect();
                    let suggestions = autocomplete::suggestions(&content, &prefix);
                    if !suggestions.is_empty() {
                        self.autocomplete = Some(AutocompleteState::new(suggestions));
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
            Message::OpenSearchResult(path, line) => {
                self.goto_line = Some(line);
                return self.handle_message(Message::SelectFile(path));
            }
            Message::SelectFile(path) => {
                self.context_menu = None;
                self.selected_path = Some(path.clone());
                if let Some(idx) = self.tabs.iter().position(|f| f.path == path) {
                    self.active_tab = Some(idx);
                    self.search_results.clear();
                    self.current_match = None;
                    if let Some(line) = self.goto_line.take() {
                        self.perform_search();
                        if let Some(pos) = self.search_results.iter().position(|(l, _)| *l == line)
                        {
                            self.current_match = Some(pos);
                            self.focus_current_match();
                        } else {
                            self.move_cursor_to(line, 0);
                        }
                    }
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
                let blocks = detect_lang(&path)
                    .and_then(|lang| blocks::parse_blocks(content.clone(), lang.to_string()))
                    .unwrap_or_default();
                let blame_path = path.clone();
                let meta = meta::read_all(&content).into_iter().next();
                let diagnostics = validate_meta_json(&content);
                file_manager::emit_open(&path);
                self.tabs.push(Tab {
                    path,
                    content,
                    editor,
                    dirty: false,
                    blame: HashMap::new(),
                    diagnostics,
                    blocks,
                    meta,
                    undo_stack: VecDeque::new(),
                    redo_stack: VecDeque::new(),
                    analysis_version: 0,
                });
                self.active_tab = Some(self.tabs.len() - 1);
                self.rename_file_name.clear();
                self.search_results.clear();
                self.current_match = None;
                if let Some(line) = self.goto_line.take() {
                    self.perform_search();
                    if let Some(pos) = self.search_results.iter().position(|(l, _)| *l == line) {
                        self.current_match = Some(pos);
                        self.focus_current_match();
                    } else {
                        self.move_cursor_to(line, 0);
                    }
                }
                return self.handle_message(Message::RunGitBlame(blame_path));
            }
            Message::FileLoaded(Err(e)) => {
                self.log.push(format!("ошибка чтения: {e}"));
                Command::none()
            }
            Message::FileContentEdited(action) => {
                if let Some(i) = self.active_tab {
                    if let Some(f) = self.tabs.get_mut(i) {
                        let is_edit = action.is_edit();
                        if is_edit {
                            push_with_limit(&mut f.undo_stack, f.content.clone());
                            f.redo_stack.clear();
                        }
                        f.editor.perform(action);
                        f.content = f.editor.text();
                        if is_edit {
                            f.dirty = true;
                        }
                        return self.schedule_analysis(i);
                    }
                }
                Command::none()
            }
            Message::Undo => {
                if let Some(f) = self.current_file_mut() {
                    if let Some(prev) = f.undo_stack.pop_back() {
                        push_with_limit(&mut f.redo_stack, f.content.clone());
                        f.content = prev;
                        f.editor = Content::with_text(&f.content);
                        if let Some(lang) = detect_lang(&f.path) {
                            if let Some(bs) =
                                blocks::parse_blocks(f.content.clone(), lang.to_string())
                            {
                                f.blocks = bs;
                            } else {
                                f.blocks.clear();
                            }
                        }
                        f.dirty = true;
                    }
                }
                Command::none()
            }
            Message::Redo => {
                if let Some(f) = self.current_file_mut() {
                    if let Some(next) = f.redo_stack.pop_back() {
                        push_with_limit(&mut f.undo_stack, f.content.clone());
                        f.content = next;
                        f.editor = Content::with_text(&f.content);
                        if let Some(lang) = detect_lang(&f.path) {
                            if let Some(bs) =
                                blocks::parse_blocks(f.content.clone(), lang.to_string())
                            {
                                f.blocks = bs;
                            } else {
                                f.blocks.clear();
                            }
                        }
                        f.dirty = true;
                    }
                }
                Command::none()
            }
            Message::AnalysisReady(path, version, blocks, diagnostics) => {
                if let Some(tab) = self.tabs.iter_mut().find(|t| t.path == path) {
                    if tab.analysis_version == version {
                        tab.blocks = blocks;
                        tab.diagnostics = diagnostics;
                    }
                }
                Command::none()
            }
            Message::NewFile => Command::none(),
            Message::SaveFile => {
                if let Some(f) = self.current_file_mut() {
                    let path = f.path.clone();
                    let mut meta = f.meta.clone().unwrap_or(VisualMeta {
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
                    });
                    meta.updated_at = Utc::now();
                    let content = meta::upsert(&f.content, &meta);
                    f.content = content.clone();
                    f.editor = Content::with_text(&f.content);
                    f.undo_stack.clear();
                    f.redo_stack.clear();
                    f.meta = Some(meta);
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
                file_manager::emit_create(&path);
                self.new_file_name.clear();
                self.tabs.push(Tab {
                    path: path.clone(),
                    content: String::new(),
                    editor: Content::new(),
                    dirty: false,
                    blame: HashMap::new(),
                    diagnostics: Vec::new(),
                    blocks: Vec::new(),
                    meta: None,
                    undo_stack: VecDeque::new(),
                    redo_stack: VecDeque::new(),
                    analysis_version: 0,
                });
                self.active_tab = Some(self.tabs.len() - 1);
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
                if let Some(i) = self.active_tab {
                    let old = self.tabs[i].path.clone();
                    self.tabs[i].path = path.clone();
                    file_manager::emit_rename(&old, &path);
                }
                self.rename_file_name.clear();
                return self.load_files(self.current_root_path().unwrap());
            }
            Message::FileRenamed(Err(e)) => {
                self.log.push(format!("ошибка переименования: {e}"));
                Command::none()
            }
            Message::RequestDeleteFile => {
                if self.active_tab.is_some() {
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
                file_manager::emit_delete(&path);
                if let Some(idx) = self.tabs.iter().position(|f| f.path == path) {
                    self.tabs.remove(idx);
                    if let Some(active) = self.active_tab {
                        if active >= idx {
                            if self.tabs.is_empty() {
                                self.active_tab = None;
                            } else {
                                self.active_tab = Some(active.saturating_sub(1));
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
                if let Some(file) = self.tabs.get(idx) {
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
                if idx < self.tabs.len() {
                    self.tabs.remove(idx);
                    if let Some(active) = self.active_tab {
                        if active >= idx {
                            if self.tabs.is_empty() {
                                self.active_tab = None;
                            } else {
                                self.active_tab = Some(active.saturating_sub(1));
                            }
                        }
                    }
                }
                Command::none()
            }
            Message::FileClosed(Ok(idx)) => {
                if idx < self.tabs.len() {
                    self.tabs.remove(idx);
                    if let Some(active) = self.active_tab {
                        if active >= idx {
                            if self.tabs.is_empty() {
                                self.active_tab = None;
                            } else {
                                self.active_tab = Some(active.saturating_sub(1));
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
            Message::StartTabDrag(index) => {
                self.tab_drag = Some(TabDragState {
                    index,
                    start: f32::NAN,
                    current: 0.0,
                });
                Command::none()
            }
            Message::UpdateTabDrag(x) => {
                if let Some(state) = self.tab_drag.as_mut() {
                    if state.start.is_nan() {
                        state.start = x;
                    }
                    state.current = x;
                }
                Command::none()
            }
            Message::EndTabDrag => {
                if let Some(state) = self.tab_drag.take() {
                    let start = if state.start.is_nan() {
                        state.current
                    } else {
                        state.start
                    };
                    let delta = state.current - start;
                    let idx = state.index;
                    let len = self.tabs.len();
                    let msg = if delta > 30.0 && idx + 1 < len {
                        Message::ReorderTab {
                            from: idx,
                            to: idx + 1,
                        }
                    } else if delta < -30.0 && idx > 0 {
                        Message::ReorderTab {
                            from: idx,
                            to: idx - 1,
                        }
                    } else {
                        Message::ActivateTab(idx)
                    };
                    return self.handle_message(msg);
                }
                Command::none()
            }
            Message::ActivateTab(idx) => {
                if idx < self.tabs.len() {
                    self.active_tab = Some(idx);
                }
                Command::none()
            }
            Message::ReorderTab { from, to } => {
                if from < self.tabs.len() && to < self.tabs.len() && from != to {
                    let tab = self.tabs.remove(from);
                    self.tabs.insert(to, tab);
                    if let Some(active) = self.active_tab {
                        self.active_tab = if active == from {
                            Some(to)
                        } else if from < active && active <= to {
                            Some(active - 1)
                        } else if to <= active && active < from {
                            Some(active + 1)
                        } else {
                            Some(active)
                        };
                    }
                }
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
            Message::ProjectSearch(query) => {
                self.search_term = query.clone();
                self.project_search_results.clear();
                if let Some(root) = self.current_root_path() {
                    Command::perform(project_search(root, query), Message::ProjectSearchFinished)
                } else {
                    Command::none()
                }
            }
            Message::ProjectSearchFinished(results) => {
                self.project_search_results = results;
                Command::none()
            }
            Message::RunParse => {
                self.loading = true;
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
                self.loading = false;
                self.log.extend(lines);
                Command::none()
            }
            Message::ParseFinished(Err(e)) => {
                self.loading = false;
                self.log.push(format!("ошибка разбора: {e}"));
                Command::none()
            }
            Message::RunLint => {
                if let Some(file) = self.current_file() {
                    let content = file.content.clone();
                    Command::perform(
                        async move {
                            let errors = viz_lint::lint_str(&content);
                            let lines: Vec<&str> = content.lines().collect();
                            let mut id_map = HashMap::new();
                            for (i, line) in lines.iter().enumerate() {
                                if let Some(idx) = line.find("@viz") {
                                    if let Some(id_pos) = line[idx..].find("id=") {
                                        let rest = &line[idx + id_pos + 3..];
                                        let id = rest
                                            .split_whitespace()
                                            .next()
                                            .unwrap_or("")
                                            .split(',')
                                            .next()
                                            .unwrap_or("")
                                            .to_string();
                                        id_map.insert(id, (i, line.len()));
                                    }
                                }
                            }
                            let mut diags = Vec::new();
                            for err in errors {
                                let ident = err
                                    .strip_prefix("узел ")
                                    .and_then(|s| s.split(':').next())
                                    .unwrap_or("<unknown>")
                                    .to_string();
                                let (line_idx, len) = id_map
                                    .get(&ident)
                                    .copied()
                                    .unwrap_or((0, lines.get(0).map(|l| l.len()).unwrap_or(0)));
                                diags.push(Diagnostic {
                                    line: line_idx,
                                    range: 0..len,
                                    message: err,
                                });
                            }
                            diags
                        },
                        Message::LintFinished,
                    )
                } else {
                    Command::none()
                }
            }
            Message::LintFinished(diags) => {
                if let Some(tab) = self.current_file_mut() {
                    tab.diagnostics = diags;
                }
                Command::none()
            }
            Message::RunGitBlame(path) => {
                match git::blame(path.to_string_lossy().as_ref()) {
                    Ok(lines) => {
                        if let Some(tab) = self.tabs.iter_mut().find(|t| t.path == path) {
                            tab.blame = lines.into_iter().map(|b| (b.line, b)).collect();
                        }
                    }
                    Err(e) => self.log.push(format!("ошибка git: {e}")),
                }
                Command::none()
            }
            Message::RunGitLog => {
                self.loading = true;
                Command::perform(
                    async move { git::log().map_err(|e| e.to_string()) },
                    Message::GitFinished,
                )
            }
            Message::GitFinished(Ok(lines)) => {
                self.loading = false;
                self.log.extend(lines);
                Command::none()
            }
            Message::GitFinished(Err(e)) => {
                self.loading = false;
                self.log.push(format!("ошибка git: {e}"));
                Command::none()
            }
            Message::RunExport => {
                self.loading = true;
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
                self.loading = false;
                self.log.extend(lines);
                Command::none()
            }
            Message::ExportFinished(Err(e)) => {
                self.loading = false;
                self.log.push(format!("ошибка экспорта: {e}"));
                Command::none()
            }
            Message::ToggleTerminal => {
                self.show_terminal = !self.show_terminal;
                Command::none()
            }
            Message::TerminalCmdChanged(cmd) => {
                self.terminal_cmd = cmd;
                Command::none()
            }
            Message::RunTerminalCmd(cmd) => {
                let cmd = cmd.trim().to_string();
                self.log.push(format!("$ {}", cmd));
                self.terminal_cmd.clear();
                if cmd == ":clear" {
                    self.log.clear();
                    return Command::none();
                }
                if cmd == ":stop" {
                    if let Some(mut child) = self.terminal_child.take() {
                        tokio::spawn(async move {
                            let _ = child.kill().await;
                        });
                    }
                    return Command::none();
                }
                let sender = self.sender.clone();
                match TokioCommand::new("sh")
                    .arg("-c")
                    .arg(cmd.clone())
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .spawn()
                {
                    Ok(mut child) => {
                        if let Some(stdout) = child.stdout.take() {
                            let mut reader = BufReader::new(stdout).lines();
                            let tx = sender.clone();
                            tokio::spawn(async move {
                                while let Ok(Some(line)) = reader.next_line().await {
                                    let _ = tx.send(line);
                                }
                            });
                        }
                        if let Some(stderr) = child.stderr.take() {
                            let mut reader = BufReader::new(stderr).lines();
                            let tx = sender.clone();
                            tokio::spawn(async move {
                                while let Ok(Some(line)) = reader.next_line().await {
                                    let _ = tx.send(line);
                                }
                            });
                        }
                        self.terminal_child = Some(child);
                    }
                    Err(e) => {
                        self.log.push(format!("ошибка запуска: {e}"));
                    }
                }
                Command::none()
            }
            Message::ShowTerminalHelp => {
                self.show_terminal_help = !self.show_terminal_help;
                Command::none()
            }
            Message::OpenDiff(left, right, ignore_ws) => {
                self.loading = true;
                let left_path = left.clone();
                let right_path = right.clone();
                Command::perform(
                    async move {
                        let left_content = fs::read_to_string(&left_path)
                            .await
                            .map_err(|e| format!("{}: {}", left_path.display(), e))?;
                        let right_content = fs::read_to_string(&right_path)
                            .await
                            .map_err(|e| format!("{}: {}", right_path.display(), e))?;
                        Ok(DiffView::new(left_content, right_content, ignore_ws))
                    },
                    Message::DiffLoaded,
                )
            }
            Message::OpenGitDiff(path, commit, ignore_ws) => {
                self.loading = true;
                let rel = if let Some(root) = self.current_root_path() {
                    path.strip_prefix(root).unwrap_or(&path).to_path_buf()
                } else {
                    path.clone()
                };
                Command::perform(
                    async move {
                        let current = fs::read_to_string(&path)
                            .await
                            .map_err(|e| format!("{}: {}", path.display(), e))?;
                        let spec = format!("{commit}:{}", rel.to_string_lossy());
                        match TokioCommand::new("git")
                            .arg("show")
                            .arg(spec)
                            .output()
                            .await
                        {
                            Ok(out) if out.status.success() => {
                                let prev = String::from_utf8_lossy(&out.stdout).to_string();
                                Ok(DiffView::new(prev, current, ignore_ws))
                            }
                            Ok(out) => {
                                let err = String::from_utf8_lossy(&out.stderr).to_string();
                                Err(err)
                            }
                            Err(e) => Err(format!("{}", e)),
                        }
                    },
                    Message::DiffLoaded,
                )
            }
            Message::DiffLoaded(Ok(diff)) => {
                self.loading = false;
                self.screen = Screen::Diff(diff);
                Command::none()
            }
            Message::DiffLoaded(Err(e)) => {
                self.loading = false;
                self.diff_error = Some(e);
                Command::none()
            }
            Message::NextDiff => {
                if let Screen::Diff(ref mut diff) = self.screen {
                    let mut all: Vec<usize> = diff
                        .left_diff
                        .iter()
                        .chain(diff.right_diff.iter())
                        .cloned()
                        .collect();
                    all.sort_unstable();
                    all.dedup();
                    if !all.is_empty() {
                        let next = all
                            .iter()
                            .find(|&&i| i > diff.current)
                            .copied()
                            .unwrap_or(all[0]);
                        diff.current = next;
                        let lines = diff.left.line_count().max(diff.right.line_count()).max(1);
                        let offset = scrollable::RelativeOffset {
                            x: 0.0,
                            y: diff.current as f32 / (lines - 1) as f32,
                        };
                        return Command::batch([
                            scrollable::snap_to(diff.left_scroll.clone(), offset),
                            scrollable::snap_to(diff.right_scroll.clone(), offset),
                        ]);
                    }
                }
                Command::none()
            }
            Message::PrevDiff => {
                if let Screen::Diff(ref mut diff) = self.screen {
                    let mut all: Vec<usize> = diff
                        .left_diff
                        .iter()
                        .chain(diff.right_diff.iter())
                        .cloned()
                        .collect();
                    all.sort_unstable();
                    all.dedup();
                    if !all.is_empty() {
                        let prev = all
                            .iter()
                            .rev()
                            .find(|&&i| i < diff.current)
                            .copied()
                            .unwrap_or(*all.last().unwrap());
                        diff.current = prev;
                        let lines = diff.left.line_count().max(diff.right.line_count()).max(1);
                        let offset = scrollable::RelativeOffset {
                            x: 0.0,
                            y: diff.current as f32 / (lines - 1) as f32,
                        };
                        return Command::batch([
                            scrollable::snap_to(diff.left_scroll.clone(), offset),
                            scrollable::snap_to(diff.right_scroll.clone(), offset),
                        ]);
                    }
                }
                Command::none()
            }
            Message::ToggleDiffIgnoreWhitespace(val) => {
                if let Screen::Diff(ref mut diff) = self.screen {
                    diff.set_ignore_whitespace(val);
                }
                Command::none()
            }
            Message::DiffError(e) => {
                self.diff_error = Some(e);
                Command::none()
            }
            Message::ClearDiffError => {
                self.diff_error = None;
                Command::none()
            }
            Message::ToggleCommandPalette => {
                self.show_command_palette = !self.show_command_palette;
                if self.show_command_palette {
                    self.query.clear();
                }
                Command::none()
            }
            Message::ExecuteCommand(cmd) => {
                self.show_command_palette = false;
                let msg = COMMANDS
                    .iter()
                    .find(|c| c.id == cmd)
                    .map(|c| c.message.clone());
                if let Some(m) = msg {
                    return self.handle_message(m);
                }
                Command::none()
            }
            Message::ToggleDir(path) => {
                self.selected_path = Some(path.clone());
                if !self.expanded_dirs.remove(&path) {
                    self.expanded_dirs.insert(path);
                }
                Command::none()
            }
            Message::NavigateUp => {
                let entries = file_manager::flatten_visible_paths(
                    &self.files,
                    &self.expanded_dirs,
                    &self.search_query,
                );
                if entries.is_empty() {
                    return Command::none();
                }
                let idx = self
                    .selected_path
                    .as_ref()
                    .and_then(|p| entries.iter().position(|e| e == p))
                    .unwrap_or(0);
                let new_idx = if idx == 0 { 0 } else { idx - 1 };
                self.selected_path = Some(entries[new_idx].clone());
                Command::none()
            }
            Message::NavigateDown => {
                let entries = file_manager::flatten_visible_paths(
                    &self.files,
                    &self.expanded_dirs,
                    &self.search_query,
                );
                if entries.is_empty() {
                    return Command::none();
                }
                let idx = self
                    .selected_path
                    .as_ref()
                    .and_then(|p| entries.iter().position(|e| e == p))
                    .unwrap_or(0);
                let new_idx = if idx + 1 >= entries.len() {
                    idx
                } else {
                    idx + 1
                };
                self.selected_path = Some(entries[new_idx].clone());
                Command::none()
            }
            Message::NavigateInto => {
                if let Some(path) = self.selected_path.clone() {
                    if let Some(entry) = file_manager::find_entry(&self.files, &path) {
                        match entry.ty {
                            EntryType::Dir => {
                                if !self.expanded_dirs.contains(&path) {
                                    self.expanded_dirs.insert(path);
                                } else if let Some(child) = entry.children.first() {
                                    self.selected_path = Some(child.path.clone());
                                }
                            }
                            EntryType::File => {
                                return self.handle_message(Message::SelectFile(path));
                            }
                        }
                    }
                }
                Command::none()
            }
            Message::NavigateBack => {
                if let Some(path) = self.selected_path.clone() {
                    if self.expanded_dirs.remove(&path) {
                        // collapsed
                    } else if let Some(parent) = path.parent() {
                        self.selected_path = Some(parent.to_path_buf());
                    }
                }
                Command::none()
            }
            Message::SearchChanged(q) => {
                self.search_query = q;
                Command::none()
            }
            Message::AddFavorite(path) => {
                if !self.favorites.contains(&path) {
                    self.favorites.push(path.clone());
                    self.settings.favorites = self.favorites.clone();
                    return Command::perform(self.settings.clone().save(), |_| {
                        Message::SettingsSaved
                    });
                }
                Command::none()
            }
            Message::RemoveFavorite(path) => {
                self.favorites.retain(|p| p != &path);
                self.settings.favorites = self.favorites.clone();
                Command::perform(self.settings.clone().save(), |_| Message::SettingsSaved)
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
                if let Ok(blocks) = serde_json::from_str::<Vec<BlockInfo>>(&ev) {
                    if let Some(f) = self.current_file_mut() {
                        f.blocks = blocks.clone();
                        if let Ok(content) = std::fs::read_to_string(&f.path) {
                            f.content = content.clone();
                            f.editor = Content::with_text(&f.content);
                        }
                    }
                    self.log.push(format!("обновлено блоков: {}", blocks.len()));
                } else {
                    self.log.push(ev);
                }
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
                for hk in self.settings.shortcuts.values() {
                    if !set.insert(hk.to_string()) {
                        self.settings_warning =
                            Some("Сочетания клавиш должны быть уникальными".into());
                        return Command::none();
                    }
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

    fn schedule_analysis(&mut self, tab_index: usize) -> Command<Message> {
        if let Some(tab) = self.tabs.get_mut(tab_index) {
            tab.analysis_version = tab.analysis_version.wrapping_add(1);
            let version = tab.analysis_version;
            let path = tab.path.clone();
            let content = tab.content.clone();
            return Command::perform(
                async move {
                    sleep(Duration::from_millis(300)).await;
                    let diagnostics = validate_meta_json(&content);
                    let blocks = detect_lang(&path)
                        .and_then(|lang| blocks::parse_blocks(content.clone(), lang.to_string()))
                        .unwrap_or_default();
                    (path, version, blocks, diagnostics)
                },
                |(path, version, blocks, diagnostics)| {
                    Message::AnalysisReady(path, version, blocks, diagnostics)
                },
            );
        }
        Command::none()
    }
}

async fn project_search(root: PathBuf, query: String) -> Vec<(PathBuf, usize, String)> {
    let mut stack = vec![root];
    let mut results = Vec::new();
    while let Some(dir) = stack.pop() {
        let mut entries = match fs::read_dir(&dir).await {
            Ok(e) => e,
            Err(_) => continue,
        };
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            let ty = match entry.file_type().await {
                Ok(t) => t,
                Err(_) => continue,
            };
            if ty.is_dir() {
                stack.push(path);
            } else if ty.is_file() {
                if let Ok(content) = fs::read_to_string(&path).await {
                    for (i, line) in content.lines().enumerate() {
                        if line.contains(&query) {
                            results.push((path.clone(), i, line.to_string()));
                        }
                    }
                }
            }
        }
    }
    results
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
