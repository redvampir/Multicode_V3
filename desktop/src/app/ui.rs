use std::ops::Range;
use std::path::PathBuf;

use iced::advanced::text::highlighter::{self, Highlighter};
use iced::widget::{button, column, container, row, scrollable, text, text_editor, MouseArea, Space};
use iced::widget::overlay::menu;
use iced::{Color, Element, Length};

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

#[derive(Debug, Clone)]
pub struct SearchHighlighter {
    matches: Vec<(usize, Range<usize>)>,
    current_line: usize,
}

impl Highlighter for SearchHighlighter {
    type Settings = Vec<(usize, Range<usize>)>;
    type Highlight = ();
    type Iterator<'a> = std::vec::IntoIter<(Range<usize>, Self::Highlight)>;

    fn new(settings: &Self::Settings) -> Self {
        Self { matches: settings.clone(), current_line: 0 }
    }

    fn update(&mut self, new_settings: &Self::Settings) {
        self.matches = new_settings.clone();
    }

    fn change_line(&mut self, line: usize) {
        self.current_line = line;
    }

    fn highlight_line(&mut self, _line: &str) -> Self::Iterator<'_> {
        let ranges = self
            .matches
            .iter()
            .filter(|(l, _)| *l == self.current_line)
            .map(|(_, range)| (range.clone(), ()))
            .collect::<Vec<_>>();
        self.current_line += 1;
        ranges.into_iter()
    }

    fn current_line(&self) -> usize {
        self.current_line
    }
}

impl MulticodeApp {
    pub fn text_editor_component(&self) -> Element<Message> {
        if let Some(file) = self.current_file() {
            if self.search_results.is_empty() {
                text_editor(&file.editor)
                    .on_action(Message::FileContentEdited)
                    .height(Length::Fill)
                    .into()
            } else {
                text_editor(&file.editor)
                    .highlight::<SearchHighlighter>(self.search_results.clone(), |_, _| {
                        highlighter::Format {
                            color: Some(Color::from_rgb(1.0, 1.0, 0.0)),
                            font: None,
                        }
                    })
                    .on_action(Message::FileContentEdited)
                    .height(Length::Fill)
                    .into()
            }
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
