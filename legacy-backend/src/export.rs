use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use crate::meta;

/// Collection of visual metadata associated with a source file.
#[derive(Debug, Serialize, Deserialize)]
pub struct VizDocument {
    /// Visual metadata entries extracted from the file.
    pub nodes: Vec<meta::VisualMeta>,
}

/// Serialize all `@VISUAL_META` comments from `content` into a `VizDocument` JSON string.
pub fn serialize_viz_document(content: &str) -> Option<String> {
    let metas = meta::read_all(content);
    if metas.is_empty() {
        None
    } else {
        serde_json::to_string(&VizDocument { nodes: metas }).ok()
    }
}

/// Deserialize a `VizDocument` from a JSON string.
pub fn deserialize_viz_document(json: &str) -> Result<VizDocument, serde_json::Error> {
    serde_json::from_str(json)
}

// Regular expressions matching different comment styles that may contain
// `@VISUAL_META` markers. Each pattern also consumes the trailing newline so
// the entire line is removed from the output.
static PYTHON_SINGLE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?m)^\s*#\s*@VISUAL_META\s*\{.*\}\s*\n?").unwrap());

static SLASH_SINGLE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?m)^\s*//\s*@VISUAL_META\s*\{.*\}\s*\n?").unwrap());

static C_STYLE_MULTI: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?ms)^\s*/\*\s*@VISUAL_META\s*\{.*?\}\s*\*/\s*\n?").unwrap());

static HTML_MULTI: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?ms)^\s*<!--\s*@VISUAL_META\s*\{.*?\}\s*-->\s*\n?").unwrap());

/// Remove all lines containing `@VISUAL_META` comments from `content`.
///
/// Supports single line comments beginning with `#` or `//` and block style
/// comments like `/* */` and `<!-- -->`.
pub fn remove_meta_lines(content: &str) -> String {
    let mut out = content.to_string();
    for re in [&PYTHON_SINGLE, &SLASH_SINGLE, &C_STYLE_MULTI, &HTML_MULTI] {
        out = re.replace_all(&out, "").to_string();
    }
    out
}

/// Prepare source content for export.
///
/// When `strip_meta` is true all `@VISUAL_META` comments are removed.
/// Otherwise the content is returned unchanged.
pub fn prepare_for_export(content: &str, strip_meta: bool) -> String {
    if strip_meta {
        remove_meta_lines(content)
    } else {
        content.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_python_comment() {
        let src = "# @VISUAL_META {\"id\":1}\nprint(\"hi\")\n";
        let cleaned = remove_meta_lines(src);
        assert!(!cleaned.contains("@VISUAL_META"));
        assert!(cleaned.contains("print"));
    }
}
