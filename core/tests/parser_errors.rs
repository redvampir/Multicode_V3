use core::parser::{parse, Lang};

#[test]
fn parse_invalid_code_returns_errors_without_panicking() {
    let cases = [
        (Lang::Rust, "fn main( {"),
        (Lang::Python, "def main(:\n    pass"),
        (Lang::JavaScript, "function main( {"),
        (Lang::Css, "body { color: }"),
        (Lang::Html, "<div>"),
        (Lang::Go, "package main\nfunc main( {"),
        (Lang::TypeScript, "function main(): {"),
    ];

    for (lang, source) in cases {
        let tree = parse(source, lang, None);
        assert!(tree.is_some(), "parser returned None for invalid source");
        let tree = tree.unwrap();
        assert!(tree.root_node().has_error(), "expected parse errors to be reported");
    }
}
