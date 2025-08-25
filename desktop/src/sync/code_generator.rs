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
/// entry, optionally inserts an `@VISUAL_META` comment using [`meta::upsert`].
/// The actual code for a block is taken from the `translations` map inside
/// [`VisualMeta`]. If a translation for the target language is missing, an
/// empty snippet is inserted which still carries the meta comment when this
/// feature is enabled.
#[derive(Debug, Clone)]
pub struct CodeGenerator {
    lang: Lang,
    insert_meta: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodeGenError {
    MissingBlocks(Vec<String>),
}

impl fmt::Display for CodeGenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodeGenError::MissingBlocks(ids) => {
                write!(f, "missing blocks for visual ids: {:?}", ids)
            }
        }
    }
}

impl std::error::Error for CodeGenError {}

impl CodeGenerator {
    /// Create a new generator for `lang`.
    ///
    /// If `insert_meta` is `true`, the generator will inject visual metadata
    /// comments into the resulting snippets. When `false`, the snippets are
    /// returned unchanged.
    pub fn new(lang: Lang, insert_meta: bool) -> Self {
        Self { lang, insert_meta }
    }

    /// Generate source code from `metas` and `blocks`.
    ///
    /// The resulting code is ordered by the `y` and `x` coordinates of
    /// [`VisualMeta`]. Only entries that have a corresponding block are
    /// included.
    pub fn generate(&self, metas: &[VisualMeta], blocks: &[Block]) -> Result<String, CodeGenError> {
        let mut metas: Vec<VisualMeta> = metas.to_vec();
        metas.sort_by(|a, b| {
            a.y.partial_cmp(&b.y)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal))
        });

        let block_map: HashMap<_, _> = blocks.iter().map(|b| (b.visual_id.clone(), b)).collect();

        let mut output = String::new();
        let lang_key = self.lang.to_string();
        let mut missing: Vec<String> = Vec::new();

        for meta in metas {
            if block_map.get(&meta.id).is_none() {
                missing.push(meta.id.clone());
                continue;
            }
            let snippet = meta
                .translations
                .get(&lang_key)
                .cloned()
                .unwrap_or_default();
            let snippet = if self.insert_meta {
                meta::upsert(&snippet, &meta)
            } else {
                snippet
            };
            if !output.is_empty() && !output.ends_with('\n') {
                output.push('\n');
            }
            output.push_str(&snippet);
            if !output.ends_with('\n') {
                output.push('\n');
            }
        }

        if !missing.is_empty() {
            tracing::warn!("Missing blocks for visual ids: {:?}", missing);
            return Err(CodeGenError::MissingBlocks(missing));
        }

        Ok(output)
    }
}

/// Format generated `code` according to indentation settings.
///
/// `indent` specifies how many indentation units should be inserted at the
/// beginning of each non-empty line. The unit itself is defined by
/// `style` — either spaces or tabs. When using spaces, `indent_width`
/// determines how many spaces make up a single unit.
pub fn format_generated_code(
    code: &str,
    indent: usize,
    style: FormattingStyle,
    indent_width: usize,
) -> String {
    let unit = match style {
        FormattingStyle::Spaces => " ".repeat(indent_width),
        FormattingStyle::Tabs => "\t".to_string(),
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
        let gen = CodeGenerator::new(lang, true);
        let out = gen.generate(&[meta.clone()], &[block]).unwrap();
        assert!(out.contains("@VISUAL_META"));
        assert!(out.contains("fn main()"));
    }

    #[test]
    fn generates_code_without_meta() {
        let lang = Lang::Rust;
        let meta = make_meta("1", "fn main() {}", lang);
        let block = dummy_block("1");
        let gen = CodeGenerator::new(lang, false);
        let out = gen.generate(&[meta.clone()], &[block]).unwrap();
        assert!(!out.contains("@VISUAL_META"));
        assert!(out.contains("fn main()"));
    }

    #[test]
    fn errors_on_missing_blocks() {
        let lang = Lang::Rust;
        let meta = make_meta("1", "fn main() {}", lang);
        let gen = CodeGenerator::new(lang, true);
        let err = gen.generate(&[meta], &[]).unwrap_err();
        assert_eq!(err, CodeGenError::MissingBlocks(vec!["1".into()]));
    }

    #[test]
    fn reports_ids_for_missing_blocks_even_when_some_exist() {
        let lang = Lang::Rust;
        let m1 = make_meta("1", "fn one() {}", lang);
        let m2 = make_meta("2", "fn two() {}", lang);
        let block = dummy_block("1");
        let gen = CodeGenerator::new(lang, true);
        let err = gen.generate(&[m1, m2], &[block]).unwrap_err();
        assert_eq!(err, CodeGenError::MissingBlocks(vec!["2".into()]));
    }

    #[test]
    fn sorts_metas_by_y_then_x() {
        let lang = Lang::Rust;
        let mut meta1 = make_meta("1", "alpha", lang);
        meta1.x = 2.0;
        meta1.y = 2.0;
        let mut meta2 = make_meta("2", "beta", lang);
        meta2.x = 3.0;
        meta2.y = 1.0;
        let mut meta3 = make_meta("3", "gamma", lang);
        meta3.x = 1.0;
        meta3.y = 1.0;

        // Metas are provided in a shuffled order.
        let metas = vec![meta1, meta2, meta3];
        let blocks = vec![dummy_block("1"), dummy_block("2"), dummy_block("3")];

        let gen = CodeGenerator::new(lang, true);
        let out = gen.generate(&metas, &blocks).unwrap();

        let pos_gamma = out.find("gamma").unwrap();
        let pos_beta = out.find("beta").unwrap();
        let pos_alpha = out.find("alpha").unwrap();
        assert!(pos_gamma < pos_beta && pos_beta < pos_alpha);
    }

    #[test]
    fn formats_code_with_indent_width_one() {
        let code = "line1\nline2";
        let formatted = format_generated_code(code, 2, FormattingStyle::Spaces, 1);
        assert_eq!(formatted, "  line1\n  line2");
    }

    #[test]
    fn formats_code_with_indent_width_two() {
        let code = "line1\nline2";
        let formatted = format_generated_code(code, 2, FormattingStyle::Spaces, 2);
        assert_eq!(formatted, "    line1\n    line2");
    }

    #[test]
    fn formats_code_with_indent_width_three() {
        let code = "line1\nline2";
        let formatted = format_generated_code(code, 2, FormattingStyle::Spaces, 3);
        assert_eq!(formatted, "      line1\n      line2");
    }

    #[test]
    fn formats_code_with_indent_width_four() {
        let code = "line1\nline2";
        let formatted = format_generated_code(code, 2, FormattingStyle::Spaces, 4);
        assert_eq!(formatted, "        line1\n        line2");
    }

    #[test]
    fn formats_code_with_tabs_ignores_indent_width() {
        let code = "line1\nline2";
        let formatted = format_generated_code(code, 2, FormattingStyle::Tabs, 4);
        assert_eq!(formatted, "\t\tline1\n\t\tline2");
    }
}
