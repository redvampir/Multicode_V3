use crate::app::ViewMode;
use crate::visual::canvas::{CanvasMessage, State as CanvasState};
use iced::widget::text_editor;

/// Main user interface state containing current view mode and editor states.
pub struct MainUI {
    /// Currently active view mode.
    pub view_mode: ViewMode,
    /// Internal state of the text editor.
    pub code_editor: text_editor::Content,
    /// Internal state of the visual editor canvas.
    pub visual_state: CanvasState,
}

impl Default for MainUI {
    fn default() -> Self {
        Self {
            view_mode: ViewMode::Code,
            code_editor: text_editor::Content::new(),
            visual_state: CanvasState::default(),
        }
    }
}

/// Messages emitted by [`MainUI`] components.
#[derive(Debug, Clone)]
pub enum MainMessage {
    /// Switch to text editor view.
    SwitchToText,
    /// Switch to visual editor view.
    SwitchToVisual,
    /// Show both editors in split view.
    SwitchToSplit,
    /// Message originating from the code editor.
    CodeEditorMsg(text_editor::Action),
    /// Message originating from the visual editor canvas.
    VisualMsg(CanvasMessage),
}
