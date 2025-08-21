use super::code_editor::markdown_preview;
use iced::advanced::Widget;

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
