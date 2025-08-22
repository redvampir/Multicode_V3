use std::{
    fs,
    path::{Path, PathBuf},
};

use once_cell::sync::Lazy;
use regex::{Error as RegexError, Regex};
use walkdir::WalkDir;

use crate::meta::VisualMeta;

static META_RE: Lazy<Result<Regex, RegexError>> =
    Lazy::new(|| Regex::new(r"@VISUAL_META\s*(\{.*?\})"));

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub file: PathBuf,
    pub line: usize,
    pub meta: VisualMeta,
}

/// Рекурсивно ищет в `root` метаданные, содержащие строку `query`.
pub fn search_metadata(root: &Path, query: &str) -> Result<Vec<SearchResult>, RegexError> {
    let re = META_RE.as_ref().map_err(|e| e.clone())?;
    let mut out = Vec::new();
    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if let Ok(content) = fs::read_to_string(path) {
            for caps in re.captures_iter(&content) {
                let json = &caps[1];
                if let Ok(meta) = serde_json::from_str::<VisualMeta>(json) {
                    if meta.id == query {
                        let start = caps.get(0).map(|m| m.start()).unwrap_or(0);
                        let line = content[..start].chars().filter(|&c| c == '\n').count() + 1;
                        out.push(SearchResult {
                            file: path.to_path_buf(),
                            line,
                            meta,
                        });
                    }
                }
            }
        }
    }
    Ok(out)
}

/// Ищет записи метаданных, ссылающиеся на идентификатор `target`.
pub fn search_links(root: &Path, target: &str) -> Result<Vec<SearchResult>, RegexError> {
    let re = META_RE.as_ref().map_err(|e| e.clone())?;
    let mut out = Vec::new();
    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if let Ok(content) = fs::read_to_string(path) {
            for caps in re.captures_iter(&content) {
                let json = &caps[1];
                if let Ok(meta) = serde_json::from_str::<VisualMeta>(json) {
                    if meta.links.iter().any(|l| l == target) {
                        let start = caps.get(0).map(|m| m.start()).unwrap_or(0);
                        let line = content[..start].chars().filter(|&c| c == '\n').count() + 1;
                        out.push(SearchResult {
                            file: path.to_path_buf(),
                            line,
                            meta,
                        });
                    }
                }
            }
        }
    }
    Ok(out)
}

/// Находит определение метаданных с указанным `id`.
pub fn goto_definition(root: &Path, id: &str) -> Result<Option<SearchResult>, RegexError> {
    Ok(search_metadata(root, id)?
        .into_iter()
        .find(|r| r.meta.id == id))
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

        let res = search_metadata(dir.path(), "one").unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].file, file1);
        assert_eq!(res[0].line, 1);

        let def = goto_definition(dir.path(), "two").unwrap().unwrap();
        assert_eq!(def.file, file2);
        assert_eq!(def.line, 1);
        assert_eq!(def.meta.id, "two");
    }

    #[test]
    fn handles_empty_and_invalid_files() {
        let dir = tempdir().unwrap();
        let empty_file = dir.path().join("empty.rs");
        let invalid_file = dir.path().join("bad.rs");
        fs::write(&empty_file, "").unwrap();
        fs::write(&invalid_file, "// @VISUAL_META {invalid}\n").unwrap();

        let res_empty = search_metadata(dir.path(), "any").unwrap();
        assert!(res_empty.is_empty());

        let res_invalid = search_metadata(dir.path(), "any").unwrap();
        assert!(res_invalid.is_empty());
    }
}
