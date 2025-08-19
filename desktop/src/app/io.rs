use std::path::{Path, PathBuf};
use iced::Command;
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

impl MulticodeApp {
    pub fn load_files(&self, root: PathBuf) -> Command<Message> {
        Command::perform(
            async move {
                task::spawn_blocking(move || {
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
                children: vec![],
            },
            FileEntry {
                path: PathBuf::from("dir"),
                ty: EntryType::Dir,
                children: vec![FileEntry {
                    path: PathBuf::from("dir/b.txt"),
                    ty: EntryType::File,
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
