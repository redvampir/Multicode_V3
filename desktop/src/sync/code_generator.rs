use multicode_core::meta::{self, VisualMeta};
use multicode_core::parser::{Block, Lang};
use std::collections::HashMap;
use std::fmt;

/// Style of indentation for formatted code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormattingStyle {
    /// Use spaces for indentation.
    Spaces,
    /// Use tabs for indentation.
    Tabs,
}

impl fmt::Display for FormattingStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormattingStyle::Spaces => f.write_str("spaces"),
            FormattingStyle::Tabs => f.write_str("tabs"),
        }
    }
}

/// Simple code generator that produces source code from visual metadata and
/// parsed blocks.
///
/// The generator looks up blocks by their `visual_id` and, for each metadata
/// entry, inserts an `@VISUAL_META` comment using [`meta::upsert`]. The actual
/// code for a block is taken from the `translations` map inside [`VisualMeta`].
/// If a translation for the target language is missing, an empty snippet is
/// inserted which still carries the meta comment.
#[derive(Debug, Clone)]
pub struct CodeGenerator {
    lang: Lang,
}

impl CodeGenerator {
    /// Create a new generator for `lang`.
    pub fn new(lang: Lang) -> Self {
        Self { lang }
    }

    /// Generate source code from `metas` and `blocks`.
    ///
    /// The resulting code is ordered by the `y` and `x` coordinates of
    /// [`VisualMeta`]. Only entries that have a corresponding block are
    /// included.
    pub fn generate(&self, metas: &[VisualMeta], blocks: &[Block]) -> String {
        let mut metas: Vec<VisualMeta> = metas.to_vec();
        metas.sort_by(|a, b| {
            a.y.partial_cmp(&b.y)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal))
        });

        let block_map: HashMap<_, _> =
            blocks.iter().map(|b| (b.visual_id.clone(), b)).collect();

        let mut output = String::new();
        let lang_key = self.lang.to_string();

        for meta in metas {
            if block_map.get(&meta.id).is_none() {
                continue;
            }
            let snippet = meta
                .translations
                .get(&lang_key)
                .cloned()
                .unwrap_or_default();
            let snippet = meta::upsert(&snippet, &meta);
            if !output.is_empty() && !output.ends_with('\n') {
                output.push('\n');
            }
            output.push_str(&snippet);
            if !output.ends_with('\n') {
                output.push('\n');
            }
        }
        output
    }
}

/// Format generated `code` according to indentation settings.
///
/// `indent` specifies how many indentation units should be inserted at the
/// beginning of each non-empty line. The unit itself is defined by
/// `style` — either spaces or tabs.
pub fn format_generated_code(code: &str, indent: usize, style: FormattingStyle) -> String {
    let unit = match style {
        FormattingStyle::Spaces => " ",
        FormattingStyle::Tabs => "\t",
    };
    let prefix = unit.repeat(indent);
    let mut lines: Vec<String> = Vec::new();
    for line in code.lines() {
        if line.trim().is_empty() {
            lines.push(String::from(line));
        } else {
            lines.push(format!("{}{}", prefix, line));
        }
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::collections::HashMap as StdHashMap;

    fn make_meta(id: &str, code: &str, lang: Lang) -> VisualMeta {
        let mut translations = StdHashMap::new();
        translations.insert(lang.to_string(), code.to_string());
        VisualMeta {
            version: 1,
            id: id.to_string(),
            x: 0.0,
            y: 0.0,
            tags: vec![],
            links: vec![],
            anchors: vec![],
            tests: vec![],
            extends: None,
            origin: None,
            translations,
            ai: None,
            extras: None,
            updated_at: Utc::now(),
        }
    }

    fn dummy_block(id: &str) -> Block {
        Block {
            visual_id: id.to_string(),
            node_id: 0,
            kind: String::new(),
            range: 0..0,
            anchors: vec![],
        }
    }

    #[test]
    fn generates_code_with_meta() {
        let lang = Lang::Rust;
        let meta = make_meta("1", "fn main() {}", lang);
        let block = dummy_block("1");
        let gen = CodeGenerator::new(lang);
        let out = gen.generate(&[meta.clone()], &[block]);
        assert!(out.contains("@VISUAL_META"));
        assert!(out.contains("fn main()"));
    }

    #[test]
    fn formats_code_with_spaces() {
        let code = "line1\nline2";
        let formatted = format_generated_code(code, 2, FormattingStyle::Spaces);
        assert_eq!(formatted, "  line1\n  line2");
    }
}

