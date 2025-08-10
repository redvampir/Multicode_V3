use std::{fs, path::{Path, PathBuf}};

use once_cell::sync::Lazy;
use regex::Regex;
use walkdir::WalkDir;

use crate::meta::VisualMeta;

static META_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"@VISUAL_META\s*(\{.*?\})").unwrap()
});

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
        if let Ok(content) = fs::read_to_string(path) {
            for caps in META_RE.captures_iter(&content) {
                let json = &caps[1];
                if let Ok(meta) = serde_json::from_str::<VisualMeta>(json) {
                    if serde_json::to_string(&meta).unwrap_or_default().contains(query) {
                        let start = caps.get(0).map(|m| m.start()).unwrap_or(0);
                        let line = content[..start].lines().count() + 1;
                        out.push(SearchResult { file: path.to_path_buf(), line, meta });
                    }
                }
            }
        }
    }
    out
}

/// Find definition of metadata with a specific `id`.
pub fn goto_definition(root: &Path, id: &str) -> Option<SearchResult> {
    search_metadata(root, id).into_iter().find(|r| r.meta.id == id)
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

