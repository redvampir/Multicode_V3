use super::code_editor::{markdown_preview, SyntaxSettings, SyntectHighlighter};
use iced::{
    advanced::{text::highlighter::Highlighter, Widget},
    Color,
};

#[test]
fn markdown_preview_renders_heading() {
    let preview = markdown_preview("# Heading");
    assert_eq!(preview.children().len(), 1);
}

#[test]
fn markdown_preview_renders_list_items() {
    let preview = markdown_preview("- item1\n- item2");
    assert_eq!(preview.children().len(), 2);
}

#[test]
fn syntect_highlighter_returns_matches_and_diagnostics() {
    let settings = SyntaxSettings {
        extension: String::from("rs"),
        matches: vec![(0, 0..4), (1, 5..9)],
        diagnostics: vec![(0, 4..5), (1, 0..4)],
        theme: String::new(),
        match_color: Color::from_rgb(1.0, 0.0, 0.0),
        diagnostic_color: Color::from_rgb(0.0, 1.0, 0.0),
    };

    let mut highlighter = SyntectHighlighter::new(&settings);

    let first_line: Vec<_> = highlighter.highlight_line("abcdefghij").collect();
    assert!(first_line.contains(&(0..4, settings.match_color)));
    assert!(first_line.contains(&(4..5, settings.diagnostic_color)));

    let second_line: Vec<_> = highlighter.highlight_line("abcdefghij").collect();
    assert!(second_line.contains(&(5..9, settings.match_color)));
    assert!(second_line.contains(&(0..4, settings.diagnostic_color)));
}
