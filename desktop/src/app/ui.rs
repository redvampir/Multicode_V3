use std::ops::Range;
use std::path::PathBuf;

use iced::advanced::text::highlighter::{self, Highlighter};
use iced::widget::{button, column, container, row, scrollable, text, text_editor, MouseArea, Space};
use iced::widget::overlay::menu;
use iced::{Color, Element, Length};
use once_cell::sync::Lazy;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

use crate::app::{EntryType, FileEntry, MulticodeApp};
use crate::app::events::Message;

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
static THEME_SET: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);

#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxSettings {
    pub extension: String,
    pub matches: Vec<(usize, Range<usize>)>,
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
        let theme = &THEME_SET.themes["InspiredGitHub"];
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
        let theme = &THEME_SET.themes["InspiredGitHub"];
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
            };
            text_editor(&file.editor)
                .highlight::<SyntectHighlighter>(settings, |c, _| {
                    highlighter::Format {
                        color: Some(*c),
                        font: None,
                    }
                })
                .on_action(Message::FileContentEdited)
                .height(Length::Fill)
                .into()
        } else {
            container(text("файл не выбран"))
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        }
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

    pub fn file_tree(&self) -> Element<Message> {
        scrollable(self.view_entries(&self.files, 0)).into()
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
