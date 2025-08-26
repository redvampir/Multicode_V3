use super::state::{Dragging, MainUI};
use crate::app::ViewMode;
use crate::sync::{ResolutionOption, SyncMessage};
use crate::visual::canvas::CanvasMessage;
use multicode_core::{export, parser::Lang, search};
use std::{path::Path, process::Command};

/// Messages emitted by [`MainUI`] components.
#[derive(Debug, Clone)]
pub enum MainMessage {
    /// Switch to text editor view.
    SwitchToText,
    /// Switch to visual editor view.
    SwitchToVisual,
    /// Show both editors in split view.
    SwitchToSplit,
    /// Search for metadata in project files.
    SearchMetadata,
    /// Export the current editor content without metadata.
    Export,
    /// Open project settings via setup script.
    OpenSettings,
    /// Message originating from the code editor.
    CodeEditorMsg(iced::widget::text_editor::Action),
    /// Start dragging a block from the palette.
    StartPaletteDrag(usize),
    /// Message originating from the visual editor canvas.
    CanvasEvent(CanvasMessage),
    /// Show conflict resolution dialog for current conflicts.
    ShowConflict,
    /// Resolve conflict with selected option. `None` closes the dialog.
    ResolveConflict(String, Option<ResolutionOption>),
}

/// Trait allowing custom message handlers to extend behaviour.
pub trait MessageHandler {
    fn handle(state: &mut MainUI, msg: MainMessage);
}

/// Default message handler implementing the current behaviour.
pub struct DefaultHandler;

impl MessageHandler for DefaultHandler {
    fn handle(state: &mut MainUI, msg: MainMessage) {
        match msg {
            MainMessage::SwitchToText => state.view_mode = ViewMode::Code,
            MainMessage::SwitchToVisual => state.view_mode = ViewMode::Schema,
            MainMessage::SwitchToSplit => state.view_mode = ViewMode::Split,
            MainMessage::SearchMetadata => {
                if let Ok(results) = search::search_metadata(Path::new("."), "id") {
                    println!("found {} metadata entries", results.len());
                }
            }
            MainMessage::Export => {
                let content = state.code_editor.text();
                if let Ok(cleaned) = export::prepare_for_export(&content, true) {
                    println!("{}", cleaned);
                }
            }
            MainMessage::OpenSettings => {
                let _ = Command::new("node").arg("scripts/setup.js").spawn();
            }
            MainMessage::CodeEditorMsg(action) => {
                state.code_editor.perform(action);
                if let Some((_code, _metas, _diag)) = state.sync_engine.handle(
                    SyncMessage::TextChanged(state.code_editor.text().to_string(), Lang::Rust),
                ) {
                    state.conflicts = state.sync_engine.last_conflicts().to_vec();
                }
            }
            MainMessage::StartPaletteDrag(i) => {
                if let Some(info) = state.palette.get(i).cloned() {
                    state.dragging = Some(Dragging::Palette(info));
                }
            }
            MainMessage::CanvasEvent(event) => match event {
                CanvasMessage::BlockDragged { index, position } => {
                    if let Some(block) = state.blocks.get_mut(index) {
                        block.x = position.x as f64;
                        block.y = position.y as f64;
                    }
                }
                CanvasMessage::Dropped { position } => {
                    if let Some(Dragging::Palette(mut info)) = state.dragging.take() {
                        info.x = position.x as f64;
                        info.y = position.y as f64;
                        state.blocks.push(info);
                    }
                }
                CanvasMessage::ConnectionCreated(conn) => {
                    if !state.connections.contains(&conn) {
                        state.connections.push(conn);
                    }
                }
                CanvasMessage::TogglePalette => {
                    state.show_palette = !state.show_palette;
                }
                _ => {}
            },
            MainMessage::ShowConflict => {
                state.active_conflict = state.conflicts.first().cloned();
            }
            MainMessage::ResolveConflict(id, Some(option)) => {
                state.sync_engine.apply_resolution(&id, option);
                state.conflicts = state.sync_engine.last_conflicts().to_vec();
                state.active_conflict = state.conflicts.first().cloned();
            }
            MainMessage::ResolveConflict(_, None) => {
                state.active_conflict = None;
            }
        }
    }
}

/// Update the [`MainUI`] using the default handler.
pub fn update(state: &mut MainUI, msg: MainMessage) {
    DefaultHandler::handle(state, msg);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_switches_mode() {
        let mut ui = MainUI::default();
        ui.view_mode = ViewMode::Schema;
        update(&mut ui, MainMessage::SwitchToText);
        assert_eq!(ui.view_mode, ViewMode::Code);
    }
}
