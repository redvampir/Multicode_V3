use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use iced::widget::overlay::menu;
use iced::widget::svg::{Handle, Svg};
use iced::widget::{button, column, row, scrollable, text, MouseArea, Space};
use iced::{Alignment, Element, Length};
use once_cell::sync::Lazy;

use crate::app::events::Message;
use crate::app::{EntryType, FileEntry};

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

fn filter_entries(entries: &[FileEntry], query: &str) -> Vec<FileEntry> {
    if query.is_empty() {
        return entries.to_vec();
    }
    let q = query.to_lowercase();
    entries
        .iter()
        .filter_map(|e| {
            let name = e
                .path
                .file_name()
                .map(|n| n.to_string_lossy().to_lowercase())?;
            match &e.ty {
                EntryType::File => {
                    if name.contains(&q) {
                        Some(e.clone())
                    } else {
                        None
                    }
                }
                EntryType::Dir => {
                    let children = filter_entries(&e.children, query);
                    if name.contains(&q) || !children.is_empty() {
                        Some(FileEntry {
                            path: e.path.clone(),
                            ty: EntryType::Dir,
                            has_meta: false,
                            children,
                        })
                    } else {
                        None
                    }
                }
            }
        })
        .collect()
}

pub fn view_entries(
    entries: &[FileEntry],
    depth: u16,
    expanded_dirs: &HashSet<PathBuf>,
    favorites: &[PathBuf],
) -> Element<'static, Message> {
    let mut rows = Vec::new();
    for entry in entries {
        let indent = Space::with_width(Length::Fixed((depth * 20) as f32));
        let is_fav = favorites.contains(&entry.path);
        let fav_icon = if is_fav { "★" } else { "☆" };
        let fav_button = button(text(fav_icon))
            .padding(0)
            .width(Length::Fixed(20.0))
            .on_press(if is_fav {
                Message::RemoveFavorite(entry.path.clone())
            } else {
                Message::AddFavorite(entry.path.clone())
            });
        match &entry.ty {
            EntryType::File => {
                let name = entry
                    .path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                let name = if entry.has_meta {
                    format!("{} ◆", name)
                } else {
                    name
                };
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
                    fav_button,
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
                let expanded = expanded_dirs.contains(&entry.path);
                let icon = if expanded { "▼" } else { "▶" };
                let content = row![text(icon), text(name)]
                    .spacing(5)
                    .align_items(Alignment::Center);
                let header = row![
                    indent,
                    fav_button,
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
                    rows.push(view_entries(
                        &entry.children,
                        depth + 1,
                        expanded_dirs,
                        favorites,
                    ));
                }
            }
        }
    }
    column(rows).into()
}

pub fn file_tree(
    entries: &[FileEntry],
    expanded_dirs: &HashSet<PathBuf>,
    search_query: &str,
    favorites: &[PathBuf],
) -> Element<'static, Message> {
    let filtered = filter_entries(entries, search_query);
    scrollable(view_entries(&filtered, 0, expanded_dirs, favorites)).into()
}
