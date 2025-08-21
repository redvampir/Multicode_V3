use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use iced::widget::overlay::menu;
use iced::widget::svg::{Handle, Svg};
use iced::widget::{button, column, row, scrollable, text, MouseArea, Space};
use iced::{Alignment, Element, Length, theme};
use once_cell::sync::Lazy;

use std::sync::Mutex;

use crate::app::events::Message;
use crate::app::{EntryType, FileEntry};

/// Trait implemented by plugins that want to react to file manager events.
///
/// All methods have default empty implementations so plugins can override
/// only the events they are interested in.
pub trait FileManagerPlugin: Send + Sync {
    /// Called after a file has been opened successfully.
    fn on_open(&self, _path: &Path) {}

    /// Called after a new file has been created.
    fn on_create(&self, _path: &Path) {}

    /// Called after a file has been deleted.
    fn on_delete(&self, _path: &Path) {}

    /// Called after a file has been renamed.
    fn on_rename(&self, _from: &Path, _to: &Path) {}
}

static FILE_MANAGER_PLUGINS: Lazy<Mutex<Vec<Box<dyn FileManagerPlugin>>>> =
    Lazy::new(|| Mutex::new(Vec::new()));

/// Register a plugin to receive file manager events.
pub fn register_plugin<P: FileManagerPlugin + 'static>(plugin: P) {
    if let Ok(mut plugins) = FILE_MANAGER_PLUGINS.lock() {
        plugins.push(Box::new(plugin));
    }
}

pub(crate) fn emit_open(path: &Path) {
    if let Ok(plugins) = FILE_MANAGER_PLUGINS.lock() {
        for p in plugins.iter() {
            p.on_open(path);
        }
    }
}

pub(crate) fn emit_create(path: &Path) {
    if let Ok(plugins) = FILE_MANAGER_PLUGINS.lock() {
        for p in plugins.iter() {
            p.on_create(path);
        }
    }
}

pub(crate) fn emit_delete(path: &Path) {
    if let Ok(plugins) = FILE_MANAGER_PLUGINS.lock() {
        for p in plugins.iter() {
            p.on_delete(path);
        }
    }
}

pub(crate) fn emit_rename(from: &Path, to: &Path) {
    if let Ok(plugins) = FILE_MANAGER_PLUGINS.lock() {
        for p in plugins.iter() {
            p.on_rename(from, to);
        }
    }
}

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
    selected: &Option<PathBuf>,
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
                let name = if let Some(name) = entry.path.file_name() {
                    let name = name.to_string_lossy().to_string();
                    if entry.has_meta {
                        format!("{} ◆", name)
                    } else {
                        name
                    }
                } else {
                    rows.push(
                        row![
                            indent,
                            fav_button,
                            button(text("Ошибка"))
                                .on_press(Message::FileError(format!(
                                    "не удалось получить имя файла: {}",
                                    entry.path.display()
                                ))),
                        ]
                        .spacing(5)
                        .align_items(Alignment::Center)
                        .height(Length::Fixed(20.0))
                        .into(),
                    );
                    continue;
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
                let mut btn = button(content).on_press(Message::SelectFile(entry.path.clone()));
                if selected.as_ref() == Some(&entry.path) {
                    btn = btn.style(theme::Button::Primary);
                }
                let row = row![
                    indent,
                    fav_button,
                    MouseArea::new(btn)
                        .on_right_press(Message::ShowContextMenu(entry.path.clone())),
                ]
                .spacing(5)
                .align_items(Alignment::Center)
                .height(Length::Fixed(20.0))
                .into();
                rows.push(row);
            }
            EntryType::Dir => {
                let name = if let Some(name) = entry.path.file_name() {
                    name.to_string_lossy().to_string()
                } else {
                    rows.push(
                        row![
                            indent,
                            fav_button,
                            button(text("Ошибка"))
                                .on_press(Message::FileError(format!(
                                    "не удалось получить имя каталога: {}",
                                    entry.path.display()
                                ))),
                        ]
                        .spacing(5)
                        .align_items(Alignment::Center)
                        .height(Length::Fixed(20.0))
                        .into(),
                    );
                    continue;
                };
                let expanded = expanded_dirs.contains(&entry.path);
                let icon = if expanded { "▼" } else { "▶" };
                let content = row![text(icon), text(name)]
                    .spacing(5)
                    .align_items(Alignment::Center);
                let mut btn = button(content).on_press(Message::ToggleDir(entry.path.clone()));
                if selected.as_ref() == Some(&entry.path) {
                    btn = btn.style(theme::Button::Primary);
                }
                let header = row![
                    indent,
                    fav_button,
                    MouseArea::new(btn)
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
                        selected,
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
    selected: &Option<PathBuf>,
) -> Element<'static, Message> {
    let filtered = filter_entries(entries, search_query);
    scrollable(view_entries(&filtered, 0, expanded_dirs, favorites, selected)).into()
}

pub fn flatten_visible_paths(
    entries: &[FileEntry],
    expanded_dirs: &HashSet<PathBuf>,
    search_query: &str,
) -> Vec<PathBuf> {
    fn flatten(entries: &[FileEntry], expanded: &HashSet<PathBuf>, out: &mut Vec<PathBuf>) {
        for e in entries {
            out.push(e.path.clone());
            if matches!(e.ty, EntryType::Dir) && expanded.contains(&e.path) {
                flatten(&e.children, expanded, out);
            }
        }
    }
    let filtered = filter_entries(entries, search_query);
    let mut out = Vec::new();
    flatten(&filtered, expanded_dirs, &mut out);
    out
}

pub fn find_entry<'a>(entries: &'a [FileEntry], path: &Path) -> Option<&'a FileEntry> {
    for e in entries {
        if e.path == path {
            return Some(e);
        }
        if matches!(e.ty, EntryType::Dir) {
            if let Some(f) = find_entry(&e.children, path) {
                return Some(f);
            }
        }
    }
    None
}
