use std::collections::HashMap;
use std::ops::Range;
use std::path::PathBuf;

use chrono::NaiveDateTime;
use iced::advanced::text::highlighter::{self, Highlighter};
use iced::widget::overlay::menu;
use iced::widget::svg::{Handle, Svg};
use iced::widget::{
    button, checkbox, column, container, row, scrollable, text, text_editor, text_input,
    tooltip::{self, Tooltip},
    MouseArea, Space,
};
use iced::{Alignment, Color, Element, Length};
use once_cell::sync::Lazy;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

use crate::app::diff::DiffView;
use crate::app::events::Message;
use crate::app::{command_palette::COMMANDS, EntryType, FileEntry, MulticodeApp};
use crate::modal::Modal;

#[derive(Debug)]
pub struct ContextMenu {
    pub path: PathBuf,
    pub state: std::cell::RefCell<menu::State>,
    pub hovered: std::cell::RefCell<Option<usize>>,
}

impl ContextMenu {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            state: std::cell::RefCell::new(menu::State::new()),
            hovered: std::cell::RefCell::new(None),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ContextMenuItem {
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

static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
pub static THEME_SET: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);

const OPEN_ICON: &[u8] = include_bytes!("../../assets/open.svg");
const SAVE_ICON: &[u8] = include_bytes!("../../assets/save.svg");
const FORMAT_ICON: &[u8] = include_bytes!("../../assets/format.svg");
const AUTOCOMPLETE_ICON: &[u8] = include_bytes!("../../assets/autocomplete.svg");
const FILE_ICON: &[u8] = include_bytes!("../../assets/file.svg");
const FILE_TEXT_ICON: &[u8] = include_bytes!("../../assets/file-text.svg");
const FILE_RUST_ICON: &[u8] = include_bytes!("../../assets/file-rust.svg");

static EXT_ICON_MAP: Lazy<HashMap<&'static str, &'static [u8]>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("rs", FILE_RUST_ICON);
    m.insert("md", FILE_TEXT_ICON);
    m.insert("txt", FILE_TEXT_ICON);
    m.insert("json", FILE_TEXT_ICON);
    m.insert("toml", FILE_TEXT_ICON);
    m
});

#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxSettings {
    pub extension: String,
    pub matches: Vec<(usize, Range<usize>)>,
    pub diagnostics: Vec<(usize, Range<usize>)>,
    pub theme: String,
}

pub struct SyntectHighlighter {
    settings: SyntaxSettings,
    highlighter: HighlightLines<'static>,
    current_line: usize,
}

impl Highlighter for SyntectHighlighter {
    type Settings = SyntaxSettings;
    type Highlight = Color;
    type Iterator<'a> = std::vec::IntoIter<(Range<usize>, Color)>;

    fn new(settings: &Self::Settings) -> Self {
        let syntax = SYNTAX_SET
            .find_syntax_by_extension(&settings.extension)
            .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());
        let theme = THEME_SET
            .themes
            .get(&settings.theme)
            .unwrap_or(&THEME_SET.themes["InspiredGitHub"]);
        Self {
            settings: settings.clone(),
            highlighter: HighlightLines::new(syntax, theme),
            current_line: 0,
        }
    }

    fn update(&mut self, new_settings: &Self::Settings) {
        let syntax = SYNTAX_SET
            .find_syntax_by_extension(&new_settings.extension)
            .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());
        let theme = THEME_SET
            .themes
            .get(&new_settings.theme)
            .unwrap_or(&THEME_SET.themes["InspiredGitHub"]);
        self.highlighter = HighlightLines::new(syntax, theme);
        self.settings = new_settings.clone();
        self.current_line = 0;
    }

    fn change_line(&mut self, line: usize) {
        self.current_line = line;
    }

    fn highlight_line(&mut self, line: &str) -> Self::Iterator<'_> {
        let mut res = Vec::new();
        if let Ok(ranges) = self.highlighter.highlight_line(line, &SYNTAX_SET) {
            let mut start = 0;
            for (style, text) in ranges {
                let len = text.len();
                let color = Color::from_rgb(
                    style.foreground.r as f32 / 255.0,
                    style.foreground.g as f32 / 255.0,
                    style.foreground.b as f32 / 255.0,
                );
                res.push((start..start + len, color));
                start += len;
            }
        }
        for (line_idx, range) in &self.settings.matches {
            if *line_idx == self.current_line {
                res.push((range.clone(), Color::from_rgb(1.0, 1.0, 0.0)));
            }
        }
        for (line_idx, range) in &self.settings.diagnostics {
            if *line_idx == self.current_line {
                res.push((range.clone(), Color::from_rgb(1.0, 0.0, 0.0)));
            }
        }
        self.current_line += 1;
        res.into_iter()
    }

    fn current_line(&self) -> usize {
        self.current_line
    }
}

impl MulticodeApp {
    pub fn search_panel_component(&self) -> Element<Message> {
        if !self.show_search_panel {
            return Space::with_height(Length::Shrink).into();
        }
        row![
            text_input("найти", &self.search_term).on_input(Message::SearchTermChanged),
            button("Найти").on_press(Message::Find),
            button("←").on_press(Message::FindPrev),
            button("→").on_press(Message::FindNext),
            text_input("заменить на", &self.replace_term).on_input(Message::ReplaceTermChanged),
            button("Заменить").on_press(Message::Replace),
            button("Заменить все").on_press(Message::ReplaceAll),
            button("×").on_press(Message::ToggleSearchPanel),
        ]
        .spacing(5)
        .into()
    }

    pub fn lint_panel_component(&self) -> Element<Message> {
        if let Some(file) = self.current_file() {
            if file.diagnostics.is_empty() {
                return Space::with_height(Length::Shrink).into();
            }
            let items = file
                .diagnostics
                .iter()
                .map(|d| text(format!("{}: {}", d.line + 1, d.message)).into())
                .collect::<Vec<Element<Message>>>();
            scrollable(column(items))
                .height(Length::Fixed(100.0))
                .into()
        } else {
            Space::with_height(Length::Shrink).into()
        }
    }

    pub fn project_search_component(&self) -> Element<Message> {
        if self.project_search_results.is_empty() {
            return Space::with_height(Length::Shrink).into();
        }
        let items = self
            .project_search_results
            .iter()
            .map(|(path, line, text)| {
                let label = format!("{}:{}: {}", path.display(), line + 1, text);
                button(label)
                    .on_press(Message::OpenSearchResult(path.clone(), *line))
                    .into()
            })
            .collect::<Vec<Element<Message>>>();
        scrollable(column(items))
            .height(Length::Fixed(150.0))
            .into()
    }

    pub fn tabs_component(&self) -> Element<Message> {
        let len = self.tabs.len();
        let tabs = self
            .tabs
            .iter()
            .enumerate()
            .map(|(i, f)| {
                let name = f
                    .path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                let tab = row![
                    button(text(name)).on_press(Message::ActivateTab(i)),
                    button(text("x")).on_press(Message::CloseFile(i))
                ]
                .spacing(5);
                MouseArea::new(tab)
                    .on_drag(move |d| {
                        if d.x > 30.0 && i + 1 < len {
                            Message::ReorderTab { from: i, to: i + 1 }
                        } else if d.x < -30.0 && i > 0 {
                            Message::ReorderTab { from: i, to: i - 1 }
                        } else {
                            Message::ActivateTab(i)
                        }
                    })
                    .into()
            })
            .collect::<Vec<Element<Message>>>();
        row(tabs).spacing(5).into()
    }

    pub fn text_editor_component(&self) -> Element<Message> {
        if let Some(file) = self.current_file() {
            let ext = file
                .path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_string();
            let settings = SyntaxSettings {
                extension: ext,
                matches: self.search_results.clone(),
                diagnostics: file
                    .diagnostics
                    .iter()
                    .map(|d| (d.line, d.range.clone()))
                    .collect(),
                theme: self.settings.syntect_theme.clone(),
            };
            let editor = text_editor(&file.editor)
                .highlight::<SyntectHighlighter>(settings, |c, _| highlighter::Format {
                    color: Some(*c),
                    font: None,
                })
                .on_action(|action| match action {
                    text_editor::Action::AddCursorUp => Message::AddCursorUp,
                    text_editor::Action::AddCursorDown => Message::AddCursorDown,
                    text_editor::Action::SelectAllMatches => Message::SelectAllMatches,
                    other => Message::FileContentEdited(other),
                });

            let editor_view: Element<_> = if self.settings.show_line_numbers {
                let lines = column(
                    (1..=file.editor.line_count())
                        .map(|i| {
                            let ln = text(i.to_string());
                            if let Some(info) = file.blame.get(&i) {
                                let date = NaiveDateTime::from_timestamp_opt(info.time, 0)
                                    .map(|dt| dt.format("%Y-%m-%d").to_string())
                                    .unwrap_or_default();
                                Tooltip::new(
                                    ln,
                                    format!("{} – {}", info.author, date),
                                    tooltip::Position::FollowCursor,
                                )
                                .into()
                            } else {
                                ln.into()
                            }
                        })
                        .collect::<Vec<Element<Message>>>(),
                );

                scrollable(
                    row![
                        container(lines).width(Length::Shrink),
                        editor.height(Length::Shrink)
                    ]
                    .spacing(5),
                )
                .height(Length::Fill)
                .into()
            } else {
                editor.height(Length::Fill).into()
            };

            let toolbar: Element<_> = if self.settings.show_toolbar {
                let open_icon = Svg::new(Handle::from_memory(OPEN_ICON))
                    .width(Length::Fixed(16.0))
                    .height(Length::Fixed(16.0));
                let save_icon = Svg::new(Handle::from_memory(SAVE_ICON))
                    .width(Length::Fixed(16.0))
                    .height(Length::Fixed(16.0));
                let format_icon = Svg::new(Handle::from_memory(FORMAT_ICON))
                    .width(Length::Fixed(16.0))
                    .height(Length::Fixed(16.0));
                let auto_icon = Svg::new(Handle::from_memory(AUTOCOMPLETE_ICON))
                    .width(Length::Fixed(16.0))
                    .height(Length::Fixed(16.0));
                let lint_btn = button("Lint").on_press(Message::RunLint);
                row![
                    button(open_icon).on_press(Message::PickFile),
                    button(save_icon).on_press(Message::SaveFile),
                    button(format_icon).on_press(Message::AutoFormat),
                    button(auto_icon).on_press(Message::AutoComplete),
                    lint_btn,
                    button("Meta").on_press(Message::ToggleMetaPanel)
                ]
                .spacing(5)
                .into()
            } else {
                Space::with_height(Length::Shrink).into()
            };
            let editor_column = column![toolbar, editor_view].spacing(5);
            if self.settings.show_markdown_preview && ext == "md" {
                use pulldown_cmark::{Event, HeadingLevel, Parser, Tag};
                let parser = Parser::new(&file.content);
                let mut elements: Vec<Element<Message>> = Vec::new();
                let mut buf = String::new();
                let mut heading = None;
                for event in parser {
                    match event {
                        Event::Start(Tag::Heading(level, _, _)) => {
                            buf.clear();
                            heading = Some(level);
                        }
                        Event::End(Tag::Heading(_, _, _)) => {
                            let size = match heading.unwrap_or(HeadingLevel::H1) {
                                HeadingLevel::H1 => 30.0,
                                HeadingLevel::H2 => 26.0,
                                HeadingLevel::H3 => 22.0,
                                _ => 20.0,
                            };
                            elements.push(text(buf.clone()).size(size).into());
                            buf.clear();
                            heading = None;
                        }
                        Event::Start(Tag::Paragraph) => buf.clear(),
                        Event::End(Tag::Paragraph) => {
                            elements.push(text(buf.clone()).into());
                            buf.clear();
                        }
                        Event::Start(Tag::Item) => buf.push_str("• "),
                        Event::End(Tag::Item) => {
                            elements.push(text(buf.clone()).into());
                            buf.clear();
                        }
                        Event::Text(t) | Event::Code(t) => buf.push_str(&t),
                        Event::SoftBreak | Event::HardBreak => buf.push('\n'),
                        _ => {}
                    }
                }
                if !buf.is_empty() {
                    elements.push(text(buf).into());
                }
                let preview = scrollable(column(elements).spacing(5)).width(Length::FillPortion(1));
                let main = row![editor_column.width(Length::FillPortion(1)), preview]
                    .spacing(5);
                if self.show_meta_panel {
                    row![
                        main.width(Length::FillPortion(3)),
                        self.meta_panel_component()
                    ]
                    .spacing(5)
                    .into()
                } else {
                    main.into()
                }
            } else {
                if self.show_meta_panel {
                    row![
                        editor_column.width(Length::FillPortion(3)),
                        self.meta_panel_component()
                    ]
                    .spacing(5)
                    .into()
                } else {
                    editor_column.into()
                }
            }
        } else {
            container(text("файл не выбран"))
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        }
    }

    pub fn diff_component(&self, diff: &DiffView) -> Element<Message> {
        let toggle = checkbox("Игнорировать пробелы", diff.ignore_whitespace)
            .on_toggle(Message::ToggleDiffIgnoreWhitespace);
        column![toggle, diff.view()].spacing(5).into()
    }

    pub fn meta_panel_component(&self) -> Element<Message> {
        if let Some(file) = self.current_file() {
            if let Some(meta) = &file.meta {
                let tags = if meta.tags.is_empty() {
                    "-".into()
                } else {
                    meta.tags.join(", ")
                };
                let links = if meta.links.is_empty() {
                    "-".into()
                } else {
                    meta.links.join(", ")
                };
                let comment = meta
                    .extras
                    .as_ref()
                    .and_then(|e| e.get("comment"))
                    .and_then(|c| c.as_str())
                    .unwrap_or("")
                    .to_string();
                column![
                    text("Мета"),
                    text(format!("Теги: {}", tags)),
                    text(format!("Связи: {}", links)),
                    text(format!("Комментарий: {}", if comment.is_empty() { "-".into() } else { comment })),
                    button("Редактировать").on_press(Message::ShowMetaDialog)
                ]
                .spacing(5)
                .width(Length::Fixed(200.0))
                .into()
            } else {
                column![
                    text("Мета отсутствует"),
                    button("Создать").on_press(Message::ShowMetaDialog)
                ]
                .spacing(5)
                .width(Length::Fixed(200.0))
                .into()
            }
        } else {
            Space::with_width(Length::Shrink).into()
        }
    }

    pub fn visual_editor_component(&self) -> Element<Message> {
        let editor = container(text("visual editor stub"))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y();
        if self.show_meta_panel {
            row![
                editor.width(Length::FillPortion(3)),
                self.meta_panel_component()
            ]
            .spacing(5)
            .into()
        } else {
            editor.into()
        }
    }

    pub fn view_entries(&self, entries: &[FileEntry], depth: u16) -> Element<Message> {
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
                    let ext = entry
                        .path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("");
                    let icon = Svg::new(Handle::from_memory(
                        EXT_ICON_MAP.get(ext).copied().unwrap_or(FILE_ICON),
                    ))
                    .width(Length::Fixed(16.0))
                    .height(Length::Fixed(16.0));
                    let content = row![icon, text(name)]
                        .spacing(5)
                        .align_items(Alignment::Center);
                    let row = row![
                        indent,
                        MouseArea::new(
                            button(content).on_press(Message::SelectFile(entry.path.clone())),
                        )
                        .on_right_press(Message::ShowContextMenu(entry.path.clone())),
                    ]
                    .spacing(5)
                    .align_items(Alignment::Center)
                    .height(Length::Fixed(20.0))
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
                    let content = row![text(icon), text(name)]
                        .spacing(5)
                        .align_items(Alignment::Center);
                    let header = row![
                        indent,
                        MouseArea::new(
                            button(content).on_press(Message::ToggleDir(entry.path.clone())),
                        )
                        .on_right_press(Message::ShowContextMenu(entry.path.clone())),
                    ]
                    .spacing(5)
                    .align_items(Alignment::Center)
                    .height(Length::Fixed(20.0))
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

    pub fn file_tree(&self) -> Element<Message> {
        scrollable(self.view_entries(&self.files, 0)).into()
    }

    pub fn status_bar_component(&self) -> Element<Message> {
        if !self.settings.show_status_bar {
            return Space::with_height(Length::Shrink).into();
        }
        if let Some(file) = self.current_file() {
            let (line, column) = file.editor.cursor_position();
            let path = file.path.to_string_lossy().to_string();
            let dirty = if file.dirty { "*" } else { "" };
            let info = format!("{}:{} | blocks {}", line + 1, column + 1, file.blocks.len());
            container(row![text(path).width(Length::Fill), text(info), text(dirty)].spacing(10))
                .width(Length::Fill)
                .padding(5)
                .into()
        } else {
            Space::with_height(Length::Shrink).into()
        }
    }

    pub fn terminal_component(&self) -> Element<Message> {
        if !self.show_terminal {
            return Space::with_height(Length::Shrink).into();
        }
        let output = scrollable(column(
            self.log
                .iter()
                .cloned()
                .map(|l| text(l).into())
                .collect::<Vec<Element<Message>>>(),
        ))
        .height(Length::Fixed(150.0));
        let input = text_input("cmd", &self.terminal_cmd)
            .on_input(Message::TerminalCmdChanged)
            .on_submit(Message::RunTerminalCmd(self.terminal_cmd.clone()));
        let clear_btn = button("Очистить").on_press(Message::RunTerminalCmd(":clear".into()));
        let stop_btn = button("Stop").on_press(Message::RunTerminalCmd(":stop".into()));
        let help_btn = button("Справка").on_press(Message::ShowTerminalHelp);
        column![
            output,
            row![input, clear_btn, stop_btn, help_btn].spacing(5)
        ]
        .spacing(5)
        .into()
    }

    pub fn error_modal(&self, content: Element<Message>) -> Element<Message> {
        if let Some(err) = &self.diff_error {
            let modal_content = container(
                column![
                    text(err.clone()),
                    button("OK").on_press(Message::ClearDiffError)
                ]
                .spacing(10),
            )
            .padding(10);
            Modal::new(content, modal_content)
                .on_blur(Message::ClearDiffError)
                .into()
        } else {
            content
        }
    }

    pub fn command_palette_modal(&self, content: Element<Message>) -> Element<Message> {
        if !self.show_command_palette {
            return content;
        }
        let query_input = text_input("команда", &self.query).on_input(Message::QueryChanged);
        let items = COMMANDS
            .iter()
            .filter(|c| c.title.to_lowercase().contains(&self.query.to_lowercase()))
            .fold(column![], |col, cmd| {
                col.push(
                    button(text(cmd.title)).on_press(Message::ExecuteCommand(cmd.id.to_string())),
                )
            })
            .spacing(5);
        let modal_content = container(column![query_input, items]).padding(10);
        Modal::new(content, modal_content)
            .on_blur(Message::ToggleCommandPalette)
            .into()
    }

    pub fn shortcuts_settings_component(&self) -> Element<Message> {
        let items = COMMANDS
            .iter()
            .map(|cmd| {
                let label = if self.shortcut_capture.as_deref() == Some(cmd.id) {
                    String::from("...")
                } else {
                    self.settings
                        .shortcuts
                        .get(cmd.id)
                        .map(|h| h.to_string())
                        .unwrap_or_else(|| String::from("-"))
                };
                row![
                    text(cmd.title),
                    button(text(label)).on_press(Message::StartCaptureShortcut(cmd.id.to_string()))
                ]
                .spacing(10)
                .into()
            })
            .collect::<Vec<Element<Message>>>();
        column(items).spacing(10).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_menu_creation() {
        let cm = ContextMenu::new(PathBuf::from("test"));
        assert!(cm.hovered.borrow().is_none());
    }
}
