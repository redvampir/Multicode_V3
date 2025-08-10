use tree_sitter::{Language, Parser, Tree};

pub fn language() -> Language {
    tree_sitter_html::LANGUAGE.into()
}

pub fn parse(source: &str) -> Option<Tree> {
    let mut parser = Parser::new();
    parser.set_language(&language()).ok()?;
    parser.parse(source, None)
}
