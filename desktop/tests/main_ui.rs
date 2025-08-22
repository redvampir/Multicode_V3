use desktop::app::ViewMode;
use desktop::ui::{MainMessage, MainUI};

#[test]
fn switch_to_text_sets_code_view_mode() {
    let mut ui = MainUI::default();
    ui.update(MainMessage::SwitchToText);
    assert_eq!(ui.view_mode, ViewMode::Code);
}

#[test]
fn switch_to_visual_sets_schema_view_mode() {
    let mut ui = MainUI::default();
    ui.update(MainMessage::SwitchToVisual);
    assert_eq!(ui.view_mode, ViewMode::Schema);
}

#[test]
fn switch_to_split_sets_split_view_mode() {
    let mut ui = MainUI::default();
    ui.update(MainMessage::SwitchToSplit);
    assert_eq!(ui.view_mode, ViewMode::Split);
}

