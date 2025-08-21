use iced::Command;
use multicode_core::meta;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::task;

use super::{EntryType, FileEntry, MulticodeApp};
use crate::app::events::Message;

pub fn pick_folder() -> impl std::future::Future<Output = Option<PathBuf>> {
    async {
        task::spawn_blocking(|| rfd::FileDialog::new().pick_folder())
            .await
            .ok()
            .flatten()
    }
}

pub fn pick_file() -> impl std::future::Future<Output = Option<PathBuf>> {
    async {
        task::spawn_blocking(|| rfd::FileDialog::new().pick_file())
            .await
            .ok()
            .flatten()
    }
}

pub fn pick_file_in_dir(dir: PathBuf) -> impl std::future::Future<Output = Option<PathBuf>> {
    async move {
        task::spawn_blocking(move || rfd::FileDialog::new().set_directory(dir).pick_file())
            .await
            .ok()
            .flatten()
    }
}

impl MulticodeApp {
    pub fn load_files(&self, root: PathBuf) -> Command<Message> {
        let tabs_meta: HashMap<PathBuf, bool> = self
            .tabs
            .iter()
            .map(|t| (t.path.clone(), t.meta.is_some()))
            .collect();
        Command::perform(
            async move {
                let tabs_meta = tabs_meta;
                task::spawn_blocking(move || {
                    fn visit(dir: &Path, tabs_meta: &HashMap<PathBuf, bool>) -> Vec<FileEntry> {
                        let mut entries = Vec::new();
                        if let Ok(read) = std::fs::read_dir(dir) {
                            let mut read: Vec<_> = read.flatten().collect();
                            read.sort_by_key(|e| e.path());
                            for entry in read {
                                if let Ok(ft) = entry.file_type() {
                                    let path = entry.path();
                                    if ft.is_dir() {
                                        let children = visit(&path, tabs_meta);
                                        entries.push(FileEntry {
                                            path,
                                            ty: EntryType::Dir,
                                            has_meta: false,
                                            children,
                                        });
                                    } else if ft.is_file() {
                                        let has_meta =
                                            tabs_meta.get(&path).copied().unwrap_or_else(|| {
                                                std::fs::read_to_string(&path)
                                                    .ok()
                                                    .map(|c| !meta::read_all(&c).is_empty())
                                                    .unwrap_or(false)
                                            });
                                        entries.push(FileEntry {
                                            path,
                                            ty: EntryType::File,
                                            has_meta,
                                            children: Vec::new(),
                                        });
                                    }
                                }
                            }
                        }
                        entries
                    }

                    visit(&root, &tabs_meta)
                })
                .await
                .map_err(|e| e.to_string())
            },
            |res| match res {
                Ok(list) => Message::FilesLoaded(list),
                Err(e) => Message::FileError(e),
            },
        )
    }

    pub fn collect_files(entries: &[FileEntry], out: &mut Vec<PathBuf>) {
        for entry in entries {
            match entry.ty {
                EntryType::File => out.push(entry.path.clone()),
                EntryType::Dir => Self::collect_files(&entry.children, out),
            }
        }
    }

    pub fn file_paths(&self) -> Vec<PathBuf> {
        let mut out = Vec::new();
        Self::collect_files(&self.files, &mut out);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collect_files_gathers_all_paths() {
        let entries = vec![
            FileEntry {
                path: PathBuf::from("a.txt"),
                ty: EntryType::File,
                has_meta: false,
                children: vec![],
            },
            FileEntry {
                path: PathBuf::from("dir"),
                ty: EntryType::Dir,
                has_meta: false,
                children: vec![FileEntry {
                    path: PathBuf::from("dir/b.txt"),
                    ty: EntryType::File,
                    has_meta: false,
                    children: vec![],
                }],
            },
        ];
        let mut out = Vec::new();
        MulticodeApp::collect_files(&entries, &mut out);
        assert_eq!(out.len(), 2);
        assert!(out.contains(&PathBuf::from("a.txt")));
        assert!(out.contains(&PathBuf::from("dir/b.txt")));
    }
}
