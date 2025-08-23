use std::{
    fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use once_cell::sync::Lazy;
use regex::{Error as RegexError, Regex};
use walkdir::WalkDir;

use crate::meta::VisualMeta;

static META_RE: Lazy<Result<Regex, RegexError>> =
    Lazy::new(|| Regex::new(r"@VISUAL_META\s*(\{.*?\})"));

/// File extensions that are searched for metadata.
const ALLOWED_EXTENSIONS: &[&str] = &["rs", "js", "ts", "tsx", "jsx"]; // extend as needed

/// Maximum file size (in bytes) to scan for metadata.
const MAX_FILE_SIZE: u64 = 1_000_000; // 1MB

fn validate_query(query: &str) -> Result<(), RegexError> {
    if query.is_empty() {
        return Err(RegexError::Syntax("query must not be empty".into()));
    }
    if !query
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(RegexError::Syntax("query contains invalid characters".into()));
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub file: PathBuf,
    pub line: usize,
    pub meta: VisualMeta,
}

/// Рекурсивно ищет в `root` метаданные с идентификатором `query`.
/// `query` должен быть непустым и состоять только из символов `[a-zA-Z0-9_-]`.
/// Возвращает ошибку, если `query` не проходит проверку.
pub fn search_metadata(root: &Path, query: &str) -> Result<Vec<SearchResult>, RegexError> {
    validate_query(query)?;
    let re = META_RE.as_ref().map_err(|e| e.clone())?;
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
                    for caps in re.captures_iter(&line) {
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
    Ok(out)
}

/// Ищет записи метаданных, ссылающиеся на идентификатор `target`.
/// `target` должен удовлетворять тем же ограничениям, что и `query`.
pub fn search_links(root: &Path, target: &str) -> Result<Vec<SearchResult>, RegexError> {
    validate_query(target)?;
    let re = META_RE.as_ref().map_err(|e| e.clone())?;
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
                    for caps in re.captures_iter(&line) {
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
    Ok(out)
}

/// Находит определение метаданных с указанным `id`.
/// `id` должен удовлетворять ограничениям, описанным для `query`.
pub fn goto_definition(root: &Path, id: &str) -> Result<Option<SearchResult>, RegexError> {
    validate_query(id)?;
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

    #[test]
    fn skips_large_and_irrelevant_files() {
        let dir = tempdir().unwrap();
        // create many irrelevant files
        for i in 0..500 {
            let p = dir.path().join(format!("file{i}.txt"));
            fs::write(p, "noop").unwrap();
        }
        // big file over size limit
        let big = dir.path().join("big.rs");
        fs::write(&big, vec![b'a'; (MAX_FILE_SIZE + 1) as usize]).unwrap();
        // file with unsupported extension containing metadata
        let ignored = dir.path().join("ignored.txt");
        fs::write(&ignored, "// @VISUAL_META {\"id\":\"one\"}\n").unwrap();
        // valid file
        let target = dir.path().join("target.rs");
        fs::write(
            &target,
            "// @VISUAL_META {\"id\":\"one\",\"x\":0,\"y\":0}\n",
        )
        .unwrap();

        let res = search_metadata(dir.path(), "one").unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].file, target);
    }

    #[test]
    fn rejects_invalid_query() {
        let dir = tempdir().unwrap();
        assert!(search_metadata(dir.path(), "").is_err());
        assert!(search_metadata(dir.path(), "bad?").is_err());
        assert!(search_links(dir.path(), "").is_err());
    }
}
