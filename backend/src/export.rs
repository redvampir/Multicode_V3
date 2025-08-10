use once_cell::sync::Lazy;
use regex::Regex;

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
