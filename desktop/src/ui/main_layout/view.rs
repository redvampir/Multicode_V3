use super::state::MainUI;
use super::update::MainMessage;
use crate::app::ViewMode;
use crate::ui::{
    conflict_dialog::{self, ConflictDialogMessage},
    sync_indicators,
};
use crate::visual::canvas::{CanvasMessage, VisualCanvas};
use iced::advanced::text::highlighter::{self, Highlighter};
use iced::widget::canvas::Canvas;
use iced::widget::{button, column, row, scrollable, text, text_editor};
use iced::{Color, Element, Length, Theme};

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
        let lines = sync_indicators::conflict_lines(&state.sync_engine);
        text_editor::<MainMessage, Theme, _>(&state.code_editor)
            .highlight::<ConflictHighlighter>(lines, |_, _| highlighter::Format {
                color: Some(Color::from_rgb(1.0, 0.0, 0.0)),
                font: None,
            })
            .on_action(MainMessage::CodeEditorMsg)
            .into()
    }
}

#[derive(Debug, Clone)]
struct ConflictHighlighter {
    lines: Vec<usize>,
    current: usize,
}

impl Highlighter for ConflictHighlighter {
    type Settings = Vec<usize>;
    type Highlight = ();
    type Iterator<'a> = std::vec::IntoIter<(std::ops::Range<usize>, ())>;

    fn new(settings: &Self::Settings) -> Self {
        Self {
            lines: settings.clone(),
            current: 0,
        }
    }

    fn update(&mut self, settings: &Self::Settings) {
        self.lines = settings.clone();
        self.current = 0;
    }

    fn change_line(&mut self, line: usize) {
        self.current = line;
    }

    fn highlight_line(&mut self, line: &str) -> Self::Iterator<'_> {
        let mut res = Vec::new();
        if self.lines.contains(&self.current) {
            res.push((0..line.len(), ()));
        }
        self.current += 1;
        res.into_iter()
    }

    fn current_line(&self) -> usize {
        self.current
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
            let palette_column =
                state
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
    vec![
        Box::new(CodeView),
        Box::new(VisualView),
        Box::new(SplitView),
    ]
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

    let status = sync_indicators::status_text(&state.sync_engine);
    let status_row = if state.conflicts.is_empty() {
        row![text(status)].spacing(10)
    } else {
        row![
            text(status),
            button("Resolve").on_press(MainMessage::ShowConflict)
        ]
        .spacing(10)
    };

    let mut layout = column![menu, status_row];

    if !state.diagnostics.orphaned_blocks.is_empty() || !state.diagnostics.unmapped_code.is_empty()
    {
        let orphaned = if state.diagnostics.orphaned_blocks.is_empty() {
            "none".to_string()
        } else {
            state.diagnostics.orphaned_blocks.join(", ")
        };
        let unmapped = if state.diagnostics.unmapped_code.is_empty() {
            "none".to_string()
        } else {
            state
                .diagnostics
                .unmapped_code
                .iter()
                .map(|r| format!("{}..{}", r.start, r.end))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let diag_text = format!("Orphaned: {} | Unmapped: {}", orphaned, unmapped);
        layout = layout.push(row![text(diag_text)].spacing(10));
    }

    layout = layout.push(content);
    if let Some(conflict) = &state.active_conflict {
        let dialog = conflict_dialog::view(conflict).map(|msg| match msg {
            ConflictDialogMessage::Resolve(choice) => {
                MainMessage::ResolveConflict(conflict.id.clone(), choice)
            }
            ConflictDialogMessage::Next => MainMessage::NextConflict,
            ConflictDialogMessage::Prev => MainMessage::PrevConflict,
        });
        layout = layout.push(dialog);
    }
    layout.into()
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
