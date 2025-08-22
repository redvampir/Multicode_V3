use crate::app::ViewMode;
use crate::visual::canvas::{CanvasMessage, State as CanvasState};
use iced::widget::{row, text, text_editor};
use iced::Element;

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

impl MainUI {
    /// Handle messages produced by the main UI and update internal state.
    pub fn update(&mut self, msg: MainMessage) {
        match msg {
            MainMessage::SwitchToText => self.view_mode = ViewMode::Code,
            MainMessage::SwitchToVisual => self.view_mode = ViewMode::Schema,
            MainMessage::SwitchToSplit => self.view_mode = ViewMode::Split,
            MainMessage::CodeEditorMsg(action) => {
                self.code_editor.perform(action);
            }
            MainMessage::VisualMsg(_msg) => {
                // Visual editor updates would be handled here once implemented
            }
        }
    }

    /// Render the current view based on the active [`ViewMode`].
    pub fn view(&self) -> Element<MainMessage> {
        match self.view_mode {
            ViewMode::Code => self.create_text_editor_view(),
            ViewMode::Schema => self.create_visual_editor_view(),
            ViewMode::Split => self.create_split_view(),
        }
    }

    /// Create a view for the text editor.
    fn create_text_editor_view(&self) -> Element<MainMessage> {
        text_editor(&self.code_editor)
            .on_action(MainMessage::CodeEditorMsg)
            .into()
    }

    /// Create a view for the visual editor (placeholder).
    fn create_visual_editor_view(&self) -> Element<MainMessage> {
        // For now, show a placeholder until the real visual editor is integrated
        text("Visual editor not implemented").into()
    }

    /// Create a split view combining text and visual editors (placeholder).
    fn create_split_view(&self) -> Element<MainMessage> {
        row![
            self.create_text_editor_view(),
            self.create_visual_editor_view()
        ]
        .into()
    }
}
