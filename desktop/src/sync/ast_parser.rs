use std::collections::HashMap;
use multicode_core::parser::{self, Block, Lang};
use multicode_core::meta::VisualMeta;

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
}

impl ASTParser {
    pub fn new(lang: Lang) -> Self {
        Self { lang }
    }
    pub fn parse(&self, code: &str, metas: &[VisualMeta]) -> SyntaxTree {
        let tree = match parser::parse(code, self.lang, None) {
            Some(t) => t,
            None => return SyntaxTree::default(),
        };
        let blocks = parser::parse_to_blocks(&tree);
        let meta_map: HashMap<_, _> = metas.iter().cloned().map(|m| (m.id.clone(), m)).collect();
        let nodes = blocks
            .into_iter()
            .map(|b| {
                let meta = meta_map.get(&b.visual_id).cloned();
                SyntaxNode { block: b, meta }
            })
            .collect();
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
        let parser = ASTParser::new(Lang::Rust);
        let m = meta("0");
        let code = multicode_core::meta::upsert("fn main() {}", &m);
        let tree = parser.parse(&code, &[m]);
        assert!(tree
            .nodes
            .iter()
            .any(|n| n.block.visual_id == "0" && n.meta.as_ref().map(|m| m.id.as_str()) == Some("0")));
    }
}
