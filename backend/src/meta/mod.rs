use serde::{Deserialize, Serialize};

/// Marker used to identify visual metadata comments in documents.
const MARKER: &str = "@VISUAL_META";

/// Metadata stored inside `@VISUAL_META` comments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualMeta {
    /// Identifier linking this metadata to AST nodes.
    pub id: String,
    /// Title of the document.
    pub title: String,
    /// List of authors.
    pub authors: Vec<String>,
    /// Optional URL for the document.
    pub url: Option<String>,
    /// Optional DOI of the document.
    pub doi: Option<String>,
    /// Optional publication date.
    pub published: Option<String>,
}

/// Insert a visual metadata comment into the given `content`.
///
/// Any existing visual metadata comment will be replaced.
pub fn insert(content: &str, meta: &VisualMeta) -> String {
    let json = match serde_json::to_string(meta) {
        Ok(j) => j,
        Err(_) => return content.to_string(),
    };

    let comment = format!("<!-- {} {} -->\n", MARKER, json);
    let content_without = remove(content);
    format!("{}{}", comment, content_without)
}

/// Read visual metadata comment from `content`.
pub fn read(content: &str) -> Option<VisualMeta> {
    let marker = format!("<!-- {}", MARKER);
    let start = content.find(&marker)?;
    let rest = &content[start + marker.len()..];
    let end = rest.find("-->")?;
    let json_str = rest[..end].trim();
    serde_json::from_str(json_str).ok()
}

/// Remove the visual metadata comment from `content` if present.
fn remove(content: &str) -> String {
    let marker = format!("<!-- {}", MARKER);
    if let Some(start) = content.find(&marker) {
        if let Some(end_rel) = content[start..].find("-->") {
            let end = start + end_rel + 3;
            let mut result = content.to_string();
            result.replace_range(start..end, "");
            return result.trim_start_matches('\n').to_string();
        }
    }
    content.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_read_roundtrip() {
        let meta = VisualMeta {
            id: "1".into(),
            title: "Example".into(),
            authors: vec!["Alice".into(), "Bob".into()],
            url: Some("https://example.com".into()),
            doi: None,
            published: Some("2024-01-01".into()),
        };

        let content = "Hello world";
        let with_meta = insert(content, &meta);
        assert!(with_meta.contains(MARKER));

        let parsed = read(&with_meta).expect("meta should be read");
        assert_eq!(parsed.title, meta.title);
        assert_eq!(parsed.authors, meta.authors);
    }
}
