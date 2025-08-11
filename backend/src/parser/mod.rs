use serde::Serialize;
use std::ops::Range;
use tree_sitter::{Language, Node, Parser, Tree};

pub mod css;
pub mod go;
pub mod html;
pub mod javascript;
pub mod python;
pub mod rust;
pub mod typescript;

/// Supported languages for parsing.
#[derive(Clone, Copy)]
pub enum Lang {
    Rust,
    Python,
    JavaScript,
    Css,
    Html,
    Go,
    TypeScript,
}

/// Get a tree-sitter [`Language`] from [`Lang`].
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

/// Parse the provided `source` using the parser for `lang`.
///
/// An optional previously parsed [`Tree`] can be supplied to enable
/// incremental parsing.
pub fn parse(source: &str, lang: Lang, old_tree: Option<&Tree>) -> Option<Tree> {
    let mut parser = Parser::new();
    parser.set_language(&language(lang)).ok()?;
    parser.parse(source, old_tree)
}

/// Block of code tied to a visual metadata identifier.
#[derive(Debug, Clone, Serialize)]
pub struct Block {
    /// Identifier linking this node with [`VisualMeta`].
    pub visual_id: String,
    /// Unique identifier of the underlying AST node.
    pub node_id: u32,
    /// Node kind as reported by tree-sitter.
    pub kind: String,
    /// Byte range of the node within the source.
    pub range: Range<usize>,
}

/// Convert an AST [`Tree`] into a flat list of [`Block`]s.
///
/// Each node in the tree is assigned a sequential `visual_id` which can later
/// be associated with a [`VisualMeta`] entry. The mapping between the tree-sitter
/// node id and `visual_id` is preserved in the returned blocks.
pub fn parse_to_blocks(tree: &Tree) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut counter: u64 = 0;

    fn walk(node: Node, blocks: &mut Vec<Block>, counter: &mut u64) {
        blocks.push(Block {
            visual_id: counter.to_string(),
            node_id: node.id() as u32,
            kind: node.kind().to_string(),
            range: node.byte_range(),
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
            let tree = parse(source, lang, None).expect("failed to parse");
            let blocks = parse_to_blocks(&tree);
            assert!(!blocks.is_empty());
            let mut unique = HashSet::new();
            for block in &blocks {
                assert!(unique.insert(block.node_id));
                assert!(!block.visual_id.is_empty());
            }
        }
    }
}
