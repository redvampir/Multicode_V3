use desktop::app::ViewMode;
use desktop::ui::{MainMessage, MainUI};
use desktop::visual::canvas::CanvasMessage;
use desktop::visual::connections::{Connection, DataType};
use iced::Point;

#[test]
fn view_mode_switching() {
    let mut ui = MainUI::default();
    ui.update(MainMessage::SwitchToVisual);
    assert_eq!(ui.view_mode, ViewMode::Schema);
    ui.update(MainMessage::SwitchToSplit);
    assert_eq!(ui.view_mode, ViewMode::Split);
}

#[test]
fn palette_drag_and_drop_adds_block() {
    let mut ui = MainUI::default();
    assert!(ui.blocks.is_empty());
    ui.update(MainMessage::StartPaletteDrag(0));
    ui.update(MainMessage::CanvasEvent(CanvasMessage::Dropped {
        position: Point::new(10.0, 20.0),
    }));
    assert_eq!(ui.blocks.len(), 1);
}

#[test]
fn connection_creation_stores_connection() {
    let mut ui = MainUI::default();
    let conn = Connection {
        from: (0, 0),
        to: (1, 0),
        data_type: DataType::Any,
    };
    ui.update(MainMessage::CanvasEvent(CanvasMessage::ConnectionCreated(
        conn,
    )));
    assert!(ui.connections.contains(&conn));
}

#[test]
fn toggle_palette_hides_and_shows() {
    let mut ui = MainUI::default();
    assert!(ui.show_palette);
    ui.update(MainMessage::CanvasEvent(CanvasMessage::TogglePalette));
    assert!(!ui.show_palette);
    ui.update(MainMessage::CanvasEvent(CanvasMessage::TogglePalette));
    assert!(ui.show_palette);
}
