use desktop::app::ViewMode;
use desktop::ui::{MainMessage, MainUI};
use desktop::visual::connections::{Connection, DataType};
use iced::Point;
use multicode_core::BlockInfo;

fn sample_block() -> BlockInfo {
    BlockInfo {
        visual_id: "1".into(),
        node_id: None,
        kind: "test".into(),
        translations: Default::default(),
        range: (0, 0),
        anchors: Vec::new(),
        x: 0.0,
        y: 0.0,
        ports: Vec::new(),
        ai: None,
        tags: Vec::new(),
        links: Vec::new(),
    }
}

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

#[test]
fn dragging_block_updates_position() {
    let mut ui = MainUI::default();
    ui.blocks.push(sample_block());
    let new_pos = Point::new(42.0, 24.0);
    ui.update(MainMessage::CanvasEvent(
        desktop::visual::canvas::CanvasMessage::BlockDragged {
            index: 0,
            position: new_pos,
        },
    ));
    assert_eq!(ui.blocks[0].x, new_pos.x as f64);
    assert_eq!(ui.blocks[0].y, new_pos.y as f64);
}

#[test]
fn duplicate_connections_are_not_added() {
    let mut ui = MainUI::default();
    let conn = Connection {
        from: (0, 0),
        to: (1, 0),
        data_type: DataType::Any,
    };
    ui.update(MainMessage::CanvasEvent(
        desktop::visual::canvas::CanvasMessage::ConnectionCreated(conn),
    ));
    ui.update(MainMessage::CanvasEvent(
        desktop::visual::canvas::CanvasMessage::ConnectionCreated(conn),
    ));
    assert_eq!(ui.connections.len(), 1);
}

#[test]
fn start_drag_with_empty_palette_does_nothing() {
    let mut ui = MainUI::default();
    ui.palette.clear();
    ui.update(MainMessage::StartPaletteDrag(0));
    assert!(ui.dragging.is_none());
}

#[test]
fn split_mode_renders_with_no_blocks() {
    let mut ui = MainUI::default();
    assert!(ui.blocks.is_empty());
    ui.update(MainMessage::SwitchToSplit);
    // should not panic when rendering
    ui.view();
    assert_eq!(ui.view_mode, ViewMode::Split);
}

