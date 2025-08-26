use multicode_core::meta::VisualMeta;
use multicode_core::parser::{self, Block, Lang};
use std::collections::HashMap;
use tree_sitter::Tree;

#[derive(Debug, Clone)]
pub struct SyntaxNode {
    pub block: Block,
    pub meta: Option<VisualMeta>,
}

#[derive(Debug, Clone, Default)]
pub struct SyntaxTree {
    pub nodes: Vec<SyntaxNode>,
}

#[derive(Debug)]
pub struct ASTParser {
    lang: Lang,
    tree: Option<Tree>,
    ids: HashMap<u32, String>,
}

impl ASTParser {
    pub fn new(lang: Lang) -> Self {
        Self {
            lang,
            tree: None,
            ids: HashMap::new(),
        }
    }

    pub fn parse(&mut self, code: &str, metas: &[VisualMeta]) -> SyntaxTree {
        let tree = match parser::parse(code, self.lang, self.tree.as_ref()) {
            Some(t) => t,
            None => return SyntaxTree::default(),
        };
        let blocks = parser::parse_to_blocks(&tree, Some(&self.ids));
        self.ids = blocks
            .iter()
            .map(|b| (b.node_id, b.visual_id.clone()))
            .collect();
        let meta_map: HashMap<_, _> = metas.iter().cloned().map(|m| (m.id.clone(), m)).collect();
        let nodes = blocks
            .into_iter()
            .map(|b| {
                let meta = meta_map.get(&b.visual_id).cloned();
                SyntaxNode { block: b, meta }
            })
            .collect();
        self.tree = Some(tree);
        SyntaxTree { nodes }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::collections::HashMap;

    fn meta(id: &str) -> VisualMeta {
        VisualMeta {
            version: 1,
            id: id.into(),
            x: 0.0,
            y: 0.0,
            tags: vec![],
            links: vec![],
            anchors: vec![],
            tests: vec![],
            extends: None,
            origin: None,
            translations: HashMap::new(),
            ai: None,
            extras: None,
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn parser_links_nodes_with_meta() {
        let mut parser = ASTParser::new(Lang::Rust);
        let m = meta("0");
        let code = multicode_core::meta::upsert("fn main() {}", &m, false);
        let tree = parser.parse(&code, &[m]);
        assert!(tree.nodes.iter().any(
            |n| n.block.visual_id == "0" && n.meta.as_ref().map(|m| m.id.as_str()) == Some("0")
        ));
    }

    #[test]
    fn preserves_visual_ids_for_unchanged_nodes() {
        let mut parser = ASTParser::new(Lang::Rust);
        let code1 = "fn main() {\n    let a = 1;\n}\n";
        let tree1 = parser.parse(code1, &[]);
        let old_map: HashMap<_, _> = tree1
            .nodes
            .iter()
            .map(|n| (n.block.node_id, n.block.visual_id.clone()))
            .collect();

        let code2 = "fn main() {\n    let a = 1;\n    let b = 2;\n}\n";
        let tree2 = parser.parse(code2, &[]);
        let mut count = 0;
        for n in &tree2.nodes {
            if let Some(id) = old_map.get(&n.block.node_id) {
                assert_eq!(id, &n.block.visual_id);
                count += 1;
            }
        }
        assert!(count > 0);
    }

    fn assert_tree_has_visual_ids(lang: Lang, code: &str) {
        let mut parser = ASTParser::new(lang);
        let tree = parser.parse(code, &[]);
        assert!(!tree.nodes.is_empty());
        assert!(tree.nodes.iter().all(|n| !n.block.visual_id.is_empty()));
    }

    #[test]
    fn parses_c_code_with_visual_ids() {
        assert_tree_has_visual_ids(Lang::C, "int main() { return 0; }");
    }

    #[test]
    fn parses_cpp_code_with_visual_ids() {
        assert_tree_has_visual_ids(Lang::Cpp, "int main() { return 0; }");
    }

    #[test]
    fn parses_java_code_with_visual_ids() {
        assert_tree_has_visual_ids(
            Lang::Java,
            "class Main { public static void main(String[] args) {} }",
        );
    }

    #[test]
    fn parses_csharp_code_with_visual_ids() {
        assert_tree_has_visual_ids(
            Lang::CSharp,
            "class Program { static void Main(string[] args) { } }",
        );
    }
}
