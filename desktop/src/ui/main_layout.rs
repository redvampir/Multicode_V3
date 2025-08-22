use iced::widget::{canvas::Canvas, container, row};
use iced::{Element, Length};

use crate::app::{events::Message as MainMessage, MulticodeApp as MainUI};
use crate::editor::CodeEditor;
use crate::visual::canvas::{CanvasMessage, VisualCanvas};
use crate::visual::connections::Connection;
use multicode_core::BlockInfo;

/// Create a view containing only the visual editor canvas.
pub fn create_visual_editor_view(state: &MainUI) -> Element<MainMessage> {
    let blocks: &[BlockInfo] = state
        .current_file()
        .map(|f| f.blocks.as_slice())
        .unwrap_or(&[]);
    let connections: &[Connection] = state
        .current_file()
        .map(|f| f.connections.as_slice())
        .unwrap_or(&[]);

    let canvas_widget = Canvas::new(VisualCanvas::new(
        blocks,
        connections,
        state.settings().language,
    ))
    .width(Length::Fill)
    .height(Length::Fill);

    let canvas: Element<CanvasMessage> = canvas_widget.into();
    canvas.map(MainMessage::CanvasEvent)
}

/// Create a view that splits the window between the text and visual editors.
pub fn create_split_view(state: &MainUI) -> Element<MainMessage> {
    let text_editor: Element<MainMessage> = CodeEditor::new(state).view();
    let visual_editor = create_visual_editor_view(state);

    row![
        container(text_editor).width(Length::FillPortion(1)),
        container(visual_editor).width(Length::FillPortion(1)),
    ]
    .spacing(10)
    .into()
}
