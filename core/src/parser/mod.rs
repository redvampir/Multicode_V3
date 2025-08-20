use serde::Serialize;
use std::fmt::{self, Display};
use std::ops::Range;
use tree_sitter::{Language, Node, Parser, Tree};

pub mod css;
pub mod go;
pub mod html;
pub mod javascript;
pub mod python;
pub mod rust;
pub mod typescript;
pub mod viz_comments;

/// Поддерживаемые языки для парсинга.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    Rust,
    Python,
    JavaScript,
    Css,
    Html,
    Go,
    TypeScript,
}

impl Display for Lang {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Lang::Rust => "rust",
            Lang::Python => "python",
            Lang::JavaScript => "javascript",
            Lang::Css => "css",
            Lang::Html => "html",
            Lang::Go => "go",
            Lang::TypeScript => "typescript",
        };
        f.write_str(name)
    }
}

impl std::str::FromStr for Lang {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "rust" => Ok(Lang::Rust),
            "python" => Ok(Lang::Python),
            "javascript" => Ok(Lang::JavaScript),
            "css" => Ok(Lang::Css),
            "html" => Ok(Lang::Html),
            "go" => Ok(Lang::Go),
            "typescript" => Ok(Lang::TypeScript),
            _ => Err(()),
        }
    }
}

/// Возвращает [`Language`] tree-sitter из [`Lang`].
fn language(lang: Lang) -> Language {
    match lang {
        Lang::Rust => rust::language(),
        Lang::Python => python::language(),
        Lang::JavaScript => javascript::language(),
        Lang::Css => css::language(),
        Lang::Html => html::language(),
        Lang::Go => go::language(),
        Lang::TypeScript => typescript::language(),
    }
}

/// Разбирает `source`, используя парсер для `lang`.
///
/// Необязательное ранее разобранное [`Tree`] позволяет выполнять
/// инкрементальный парсинг.
pub fn parse(source: &str, lang: Lang, old_tree: Option<&Tree>) -> Option<Tree> {
    let mut parser = Parser::new();
    parser.set_language(&language(lang)).ok()?;
    parser.parse(source, old_tree)
}

/// Блок кода, связанный с идентификатором визуальных метаданных.
#[derive(Debug, Clone, Serialize)]
pub struct Block {
    /// Идентификатор, связывающий этот узел с [`VisualMeta`].
    pub visual_id: String,
    /// Уникальный идентификатор соответствующего узла AST.
    pub node_id: u32,
    /// Тип узла, сообщаемый tree-sitter.
    pub kind: String,
    /// Байтовый диапазон узла в исходнике.
    pub range: Range<usize>,
    /// Якоря, указывающие на диапазоны в исходном коде.
    pub anchors: Vec<(usize, usize)>,
}

/// Преобразует AST [`Tree`] в плоский список [`Block`].
///
/// Каждому узлу дерева присваивается последовательный `visual_id`, который
/// позже может быть связан с записью [`VisualMeta`]. Соответствие между
/// идентификатором узла tree-sitter и `visual_id` сохраняется в возвращаемых блоках.
pub fn parse_to_blocks(tree: &Tree) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut counter: u64 = 0;

    fn map_kind(kind: &str) -> String {
        match kind {
            "+" | "-" | "*" | "/" | "%" | "&&" | "||" | "==" | "!=" | ">" | ">=" | "<" | "<=" => {
                format!("Op/{kind}")
            }
            "?" => "Op/Ternary".into(),
            "identifier" => "Variable/Get".into(),
            _ => {
                let k = kind.to_lowercase();
                if k.contains("call") && !k.contains("function") {
                    "Function/Call".into()
                } else if k.contains("return") {
                    "Return".into()
                } else if k.contains("function") || k.contains("method") {
                    "Function/Define".into()
                } else {
                    kind.to_string()
                }
            }
        }
    }

    fn walk(node: Node, blocks: &mut Vec<Block>, counter: &mut u64) {
        let range = node.byte_range();
        let kind = map_kind(node.kind());
        let anchors = if kind.starts_with("Op/") || kind == "Variable/Get" {
            vec![(range.start, range.end)]
        } else {
            vec![]
        };
        blocks.push(Block {
            visual_id: counter.to_string(),
            node_id: node.id() as u32,
            kind,
            range,
            anchors,
        });
        *counter += 1;

        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                let child = cursor.node();
                walk(child, blocks, counter);
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
    }

    walk(tree.root_node(), &mut blocks, &mut counter);
    blocks
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn lang_display_and_from_str() {
        let cases = [
            (Lang::Rust, "rust"),
            (Lang::Python, "python"),
            (Lang::JavaScript, "javascript"),
            (Lang::Css, "css"),
            (Lang::Html, "html"),
            (Lang::Go, "go"),
            (Lang::TypeScript, "typescript"),
        ];
        for (lang, name) in cases {
            assert_eq!(lang.to_string(), name);
            assert_eq!(name.parse::<Lang>().ok(), Some(lang));
        }
    }

    #[test]
    fn parse_sources_into_blocks() {
        let cases = [
            (Lang::Rust, "fn main() { println!(\"hi\"); }"),
            (Lang::Python, "def main():\n    print('hi')"),
            (Lang::JavaScript, "function main() { console.log('hi'); }"),
            (Lang::Css, "body { color: red; }"),
            (Lang::Html, "<html></html>"),
            (Lang::Go, "package main\nfunc main() { println(\"hi\") }"),
            (
                Lang::TypeScript,
                "function main(): void { console.log('hi'); }",
            ),
        ];

        for (lang, source) in cases {
            let tree = parse(source, lang, None).expect("не удалось разобрать");
            let blocks = parse_to_blocks(&tree);
            assert!(!blocks.is_empty());
            let mut unique = HashSet::new();
            for block in &blocks {
                assert!(unique.insert(block.node_id));
                assert!(!block.visual_id.is_empty());
            }
        }
    }

    #[test]
    fn parse_expression_into_ops_and_variables() {
        let src = "a + b * c";
        let tree = parse(src, Lang::Python, None).expect("не удалось разобрать");
        let blocks = parse_to_blocks(&tree);
        let mut found = 0;
        for b in &blocks {
            match b.kind.as_str() {
                "Op/+" => {
                    assert_eq!(b.anchors, vec![(2, 3)]);
                    found += 1;
                }
                "Op/*" => {
                    assert_eq!(b.anchors, vec![(6, 7)]);
                    found += 1;
                }
                "Variable/Get" => {
                    if b.anchors == vec![(0, 1)]
                        || b.anchors == vec![(4, 5)]
                        || b.anchors == vec![(8, 9)]
                    {
                        found += 1;
                    }
                }
                _ => {}
            }
        }
        assert_eq!(found, 5);
    }

    #[test]
    fn parse_ternary_expression_into_op() {
        let src = "a ? b : c";
        let tree = parse(src, Lang::JavaScript, None).expect("не удалось разобрать");
        let blocks = parse_to_blocks(&tree);
        assert!(blocks.iter().any(|b| b.kind == "Op/Ternary"));
    }
}
