use super::{
    code_editor::markdown_preview,
    syntax_highlighter::{SyntaxColors, SyntaxHighlighter, SyntaxSettings},
};
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
        colors: SyntaxColors {
            match_color: Color::from_rgb(1.0, 0.0, 0.0),
            diagnostic_color: Color::from_rgb(0.0, 1.0, 0.0),
            meta_color: Color::from_rgb(0.0, 0.0, 1.0),
        },
    };

    let mut highlighter = SyntaxHighlighter::new(&settings);

    let first_line: Vec<_> = highlighter.highlight_line("abcdefghij").collect();
    assert!(first_line.contains(&(0..4, settings.colors.match_color)));
    assert!(first_line.contains(&(4..5, settings.colors.diagnostic_color)));

    let second_line: Vec<_> = highlighter.highlight_line("abcdefghij").collect();
    assert!(second_line.contains(&(5..9, settings.colors.match_color)));
    assert!(second_line.contains(&(0..4, settings.colors.diagnostic_color)));
}

#[test]
fn syntax_highlighter_highlights_meta_comments() {
    let settings = SyntaxSettings {
        extension: String::from("rs"),
        matches: vec![],
        diagnostics: vec![],
        theme: String::new(),
        colors: SyntaxColors {
            match_color: Color::BLACK,
            diagnostic_color: Color::BLACK,
            meta_color: Color::from_rgb(0.5, 0.0, 0.5),
        },
    };
    let mut highlighter = SyntaxHighlighter::new(&settings);
    let line: Vec<_> = highlighter
        .highlight_line("// @VISUAL_META {\"id\":1}")
        .collect();
    assert!(line.iter().any(|(_, c)| *c == settings.colors.meta_color));
}
