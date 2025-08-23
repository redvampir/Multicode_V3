use super::state::MainUI;
use super::update::MainMessage;
use crate::app::ViewMode;
use crate::visual::canvas::{CanvasMessage, VisualCanvas};
use iced::widget::canvas::Canvas;
use iced::widget::{button, column, row, scrollable, text, text_editor};
use iced::{Element, Length};

/// Trait describing a visual mode renderer.
pub trait ModeView {
    /// Identifier of the mode this renderer supports.
    fn mode(&self) -> ViewMode;
    /// Render the view for the given [`MainUI`] state.
    fn view<'a>(&self, state: &'a MainUI) -> Element<'a, MainMessage>;
}

pub struct CodeView;

impl ModeView for CodeView {
    fn mode(&self) -> ViewMode {
        ViewMode::Code
    }

    fn view<'a>(&self, state: &'a MainUI) -> Element<'a, MainMessage> {
        text_editor(&state.code_editor)
            .on_action(MainMessage::CodeEditorMsg)
            .into()
    }
}

pub struct VisualView;

impl ModeView for VisualView {
    fn mode(&self) -> ViewMode {
        ViewMode::Schema
    }

    fn view<'a>(&self, state: &'a MainUI) -> Element<'a, MainMessage> {
        let canvas_widget = Canvas::new(VisualCanvas::new(
            &state.blocks,
            &state.connections,
            state.language,
        ))
        .width(Length::Fill)
        .height(Length::Fill);
        let canvas: Element<CanvasMessage> = canvas_widget.into();
        let canvas = canvas.map(MainMessage::CanvasEvent);
        if state.show_palette {
            let palette_column = state
                .palette
                .iter()
                .enumerate()
                .fold(column!().spacing(5), |col, (i, b)| {
                    col.push(button(text(&b.kind)).on_press(MainMessage::StartPaletteDrag(i)))
                });
            let palette = scrollable(palette_column)
                .width(Length::Fixed(150.0))
                .height(Length::Fill);
            row![palette, canvas].into()
        } else {
            canvas
        }
    }
}

pub struct SplitView;

impl ModeView for SplitView {
    fn mode(&self) -> ViewMode {
        ViewMode::Split
    }

    fn view<'a>(&self, state: &'a MainUI) -> Element<'a, MainMessage> {
        row![CodeView.view(state), VisualView.view(state)].into()
    }
}

/// Default set of view mode renderers bundled with the application.
pub fn default_modes() -> Vec<Box<dyn ModeView>> {
    vec![Box::new(CodeView), Box::new(VisualView), Box::new(SplitView)]
}

/// Render the current view based on the active [`ViewMode`].
pub fn view<'a>(state: &'a MainUI) -> Element<'a, MainMessage> {
    let content = if let Some(mode) = state
        .view_modes
        .iter()
        .find(|m| m.mode() == state.view_mode)
    {
        mode.view(state)
    } else {
        text("Unsupported mode").into()
    };

    let menu = row![
        button("Text").on_press(MainMessage::SwitchToText),
        button("Visual").on_press(MainMessage::SwitchToVisual),
        button("Split").on_press(MainMessage::SwitchToSplit),
        button("Meta search").on_press(MainMessage::SearchMetadata),
        button("Export").on_press(MainMessage::Export),
        button("Settings").on_press(MainMessage::OpenSettings)
    ]
    .spacing(10);

    column![menu, content].into()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyView;
    impl ModeView for DummyView {
        fn mode(&self) -> ViewMode {
            ViewMode::Code
        }
        fn view<'a>(&self, _state: &'a MainUI) -> Element<'a, MainMessage> {
            text("dummy").into()
        }
    }

    #[test]
    fn custom_view_mode_can_be_registered() {
        let mut ui = MainUI::default();
        let initial = ui.view_modes.len();
        ui.view_modes.push(Box::new(DummyView));
        assert_eq!(ui.view_modes.len(), initial + 1);
    }
}
