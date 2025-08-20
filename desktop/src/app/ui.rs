use std::collections::HashMap;
use std::ops::Range;
use std::path::PathBuf;

use iced::advanced::text::highlighter::{self, Highlighter};
use iced::widget::overlay::menu;
use iced::widget::svg::{Handle, Svg};
use iced::widget::{
    button, column, container, row, scrollable, text, text_editor, text_input, MouseArea, Space,
};
use iced::{Alignment, Color, Element, Length};
use once_cell::sync::Lazy;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

use crate::app::diff::DiffView;
use crate::app::events::Message;
use crate::app::{EntryType, FileEntry, MulticodeApp};

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
                theme: self.settings.syntect_theme.clone(),
            };
            let editor = text_editor(&file.editor)
                .highlight::<SyntectHighlighter>(settings, |c, _| highlighter::Format {
                    color: Some(*c),
                    font: None,
                })
                .on_action(Message::FileContentEdited);

            let editor_view: Element<_> = if self.settings.show_line_numbers {
                let lines = column(
                    (1..=file.editor.line_count())
                        .map(|i| text(i.to_string()).into())
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
                row![
                    button(open_icon).on_press(Message::PickFile),
                    button(save_icon).on_press(Message::SaveFile),
                    button(format_icon).on_press(Message::AutoFormat),
                    button(auto_icon).on_press(Message::AutoComplete)
                ]
                .spacing(5)
                .into()
            } else {
                Space::with_height(Length::Shrink).into()
            };

            column![toolbar, editor_view].spacing(5).into()
        } else {
            container(text("файл не выбран"))
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        }
    }

    pub fn diff_component(&self, diff: &DiffView) -> Element<Message> {
        diff.view()
    }

    pub fn visual_editor_component(&self) -> Element<Message> {
        container(text("visual editor stub"))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
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
            container(
                row![
                    text(path).width(Length::Fill),
                    text(format!("{}:{}", line + 1, column + 1)),
                    text(dirty)
                ]
                .spacing(10),
            )
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
