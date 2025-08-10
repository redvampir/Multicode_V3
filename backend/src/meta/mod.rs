use serde::{Deserialize, Serialize};

/// Marker used to identify visual metadata comments in documents.
const MARKER: &str = "@VISUAL_META";

/// Metadata stored inside `@VISUAL_META` comments.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Translations {
    pub ru: Option<String>,
    pub en: Option<String>,
    pub es: Option<String>,
}

/// Metadata stored inside `@VISUAL_META` comments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualMeta {
    /// Identifier linking this metadata to AST nodes.
    pub id: String,
    /// X coordinate on the canvas.
    pub x: f64,
    /// Y coordinate on the canvas.
    pub y: f64,
    /// Optional translations for block labels.
    #[serde(default)]
    pub translations: Translations,
}

/// Insert or update a visual metadata comment in `content`.
///
/// The comment will be placed at the top of the document if it does not exist.
pub fn upsert(content: &str, meta: &VisualMeta) -> String {
    let marker = format!("<!-- {} ", MARKER);
    let serialized = match serde_json::to_string(meta) {
        Ok(s) => s,
        Err(_) => return content.to_string(),
    };

    let mut out = String::new();
    let mut found = false;
    for line in content.lines() {
        if line.trim_start().starts_with(&marker) {
            if let Some(end_idx) = line.find("-->") {
                let json_part = &line[marker.len()..end_idx].trim();
                if let Ok(existing) = serde_json::from_str::<VisualMeta>(json_part) {
                    if existing.id == meta.id {
                        out.push_str(&format!("{}{} -->\n", marker, serialized));
                        found = true;
                        continue;
                    }
                }
            }
        }
        out.push_str(line);
        out.push('\n');
    }

    if !found {
        out = format!("{}{} -->\n{}", marker, serialized, out);
    }

    out
}

/// Read all visual metadata comments from `content`.
pub fn read_all(content: &str) -> Vec<VisualMeta> {
    let marker = format!("<!-- {} ", MARKER);
    let mut metas = Vec::new();
    for line in content.lines() {
        if let Some(start) = line.find(&marker) {
            if let Some(end_idx) = line[start + marker.len()..].find("-->") {
                let json_str = &line[start + marker.len()..start + marker.len() + end_idx];
                if let Ok(meta) = serde_json::from_str::<VisualMeta>(json_str.trim()) {
                    metas.push(meta);
                }
            }
        }
    }
    metas
}

/// Remove all visual metadata comments from `content`.
pub fn remove_all(content: &str) -> String {
    let marker = format!("<!-- {} ", MARKER);
    let mut out = String::new();
    for line in content.lines() {
        if !line.trim_start().starts_with(&marker) {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upsert_and_read_roundtrip() {
        let meta = VisualMeta { id: "1".into(), x: 10.0, y: 20.0, translations: Translations::default() };
        let content = "fn main() {}";
        let updated = upsert(content, &meta);
        assert!(updated.contains(MARKER));
        let metas = read_all(&updated);
        assert_eq!(metas.len(), 1);
        assert_eq!(metas[0].x, 10.0);
    }

    #[test]
    fn remove_all_strips_metadata() {
        let content = format!(
            "line1\n<!-- {} {{\"id\":\"1\"}} -->\nline2\n",
            MARKER
        );
        let cleaned = remove_all(&content);
        assert!(!cleaned.contains(MARKER));
        assert!(cleaned.contains("line1"));
        assert!(cleaned.contains("line2"));
    }
}
