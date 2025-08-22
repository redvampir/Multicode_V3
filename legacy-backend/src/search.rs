use std::{
    fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use once_cell::sync::Lazy;
use regex::Regex;
use walkdir::WalkDir;

use crate::meta::VisualMeta;

static META_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"@VISUAL_META\s*(\{.*?\})").unwrap());

/// File extensions that are searched for metadata.
const ALLOWED_EXTENSIONS: &[&str] = &["rs", "js", "ts", "tsx", "jsx"]; // extend as needed

/// Maximum file size (in bytes) to scan for metadata.
const MAX_FILE_SIZE: u64 = 1_000_000; // 1MB

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub file: PathBuf,
    pub line: usize,
    pub meta: VisualMeta,
}

/// Recursively search `root` for metadata containing `query` string.
pub fn search_metadata(root: &Path, query: &str) -> Vec<SearchResult> {
    let mut out = Vec::new();
    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if !path
            .extension()
            .and_then(|e| e.to_str())
            .map_or(false, |e| ALLOWED_EXTENSIONS.contains(&e))
        {
            continue;
        }
        if let Ok(meta) = entry.metadata() {
            if meta.len() > MAX_FILE_SIZE {
                continue;
            }
        }
        if let Ok(file) = fs::File::open(path) {
            let reader = BufReader::new(file);
            for (idx, line) in reader.lines().enumerate() {
                if let Ok(line) = line {
                    for caps in META_RE.captures_iter(&line) {
                        let json = &caps[1];
                        if let Ok(meta) = serde_json::from_str::<VisualMeta>(json) {
                            if meta.id == query {
                                out.push(SearchResult {
                                    file: path.to_path_buf(),
                                    line: idx + 1,
                                    meta,
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    out
}

/// Search for metadata entries linking to `target` id.
pub fn search_links(root: &Path, target: &str) -> Vec<SearchResult> {
    let mut out = Vec::new();
    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if !path
            .extension()
            .and_then(|e| e.to_str())
            .map_or(false, |e| ALLOWED_EXTENSIONS.contains(&e))
        {
            continue;
        }
        if let Ok(meta) = entry.metadata() {
            if meta.len() > MAX_FILE_SIZE {
                continue;
            }
        }
        if let Ok(file) = fs::File::open(path) {
            let reader = BufReader::new(file);
            for (idx, line) in reader.lines().enumerate() {
                if let Ok(line) = line {
                    for caps in META_RE.captures_iter(&line) {
                        let json = &caps[1];
                        if let Ok(meta) = serde_json::from_str::<VisualMeta>(json) {
                            if meta.links.iter().any(|l| l == target) {
                                out.push(SearchResult {
                                    file: path.to_path_buf(),
                                    line: idx + 1,
                                    meta,
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    out
}

/// Find definition of metadata with a specific `id`.
pub fn goto_definition(root: &Path, id: &str) -> Option<SearchResult> {
    search_metadata(root, id)
        .into_iter()
        .find(|r| r.meta.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn search_and_goto_work() {
        let dir = tempdir().unwrap();
        let file1 = dir.path().join("a.rs");
        let file2 = dir.path().join("b.rs");
        fs::write(&file1, "// @VISUAL_META {\"id\":\"one\",\"x\":0,\"y\":0}\n").unwrap();
        fs::write(&file2, "// @VISUAL_META {\"id\":\"two\",\"x\":0,\"y\":0}\n").unwrap();

        let res = search_metadata(dir.path(), "one");
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].file, file1);
        assert_eq!(res[0].line, 1);

        let def = goto_definition(dir.path(), "two").unwrap();
        assert_eq!(def.file, file2);
        assert_eq!(def.line, 1);
        assert_eq!(def.meta.id, "two");
    }
}
