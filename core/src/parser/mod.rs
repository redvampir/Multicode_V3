use serde::Serialize;
use std::collections::HashMap;
use std::fmt::{self, Display};
use std::ops::Range;
use tree_sitter::{Language, Node, Parser, Tree};

#[macro_export]
macro_rules! define_lang_parser {
    ($lang:expr) => {
        use tree_sitter::{Language, Parser, Tree};

        pub fn language() -> Language {
            $lang.into()
        }

        pub fn parse(source: &str, old_tree: Option<&Tree>) -> Option<Tree> {
            let mut parser = Parser::new();
            parser.set_language(&language()).ok()?;
            parser.parse(source, old_tree)
        }
    };
}

pub mod c;
pub mod c_sharp;
pub mod cpp;
pub mod css;
pub mod go;
pub mod html;
pub mod java;
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
    C,
    Cpp,
    Java,
    CSharp,
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
            Lang::C => "c",
            Lang::Cpp => "cpp",
            Lang::Java => "java",
            Lang::CSharp => "csharp",
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
            "c" => Ok(Lang::C),
            "cpp" | "c++" => Ok(Lang::Cpp),
            "java" => Ok(Lang::Java),
            "csharp" | "c#" => Ok(Lang::CSharp),
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
        Lang::C => c::language(),
        Lang::Cpp => cpp::language(),
        Lang::Java => java::language(),
        Lang::CSharp => c_sharp::language(),
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
pub fn parse_to_blocks(tree: &Tree, prev: Option<&HashMap<u32, String>>) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut counter: u64 = prev
        .and_then(|m| {
            m.values()
                .filter_map(|v| v.parse::<u64>().ok())
                .max()
                .map(|m| m + 1)
        })
        .unwrap_or(0);

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

    fn walk(
        node: Node,
        blocks: &mut Vec<Block>,
        counter: &mut u64,
        prev: Option<&HashMap<u32, String>>,
    ) {
        let range = node.byte_range();
        let kind = map_kind(node.kind());
        let anchors = if kind.starts_with("Op/") || kind == "Variable/Get" {
            vec![(range.start, range.end)]
        } else {
            vec![]
        };

        let node_id = node.id() as u32;
        let visual_id = if let Some(map) = prev {
            if let Some(id) = map.get(&node_id) {
                id.clone()
            } else {
                let id = counter.to_string();
                *counter += 1;
                id
            }
        } else {
            let id = counter.to_string();
            *counter += 1;
            id
        };

        blocks.push(Block {
            visual_id,
            node_id,
            kind,
            range,
            anchors,
        });

        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                let child = cursor.node();
                walk(child, blocks, counter, prev);
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
    }

    walk(tree.root_node(), &mut blocks, &mut counter, prev);
    blocks
}

#[cfg(test)]
mod tests;
