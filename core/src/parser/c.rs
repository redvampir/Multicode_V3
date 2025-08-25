use tree_sitter::{Language, Parser, Tree};

pub fn language() -> Language {
    tree_sitter_c::LANGUAGE.into()
}

pub fn parse(source: &str, old_tree: Option<&Tree>) -> Option<Tree> {
    let mut parser = Parser::new();
    parser.set_language(&language()).ok()?;
    parser.parse(source, old_tree)
}
