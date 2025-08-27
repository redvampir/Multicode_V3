use super::state::{Dragging, MainUI};
use crate::app::ViewMode;
use crate::sync::{ResolutionOption, SyncMessage};
use crate::visual::canvas::CanvasMessage;
use crate::visual::connections::{Connection, DataType};
use multicode_core::{export, search, BlockInfo};
use multicode_core::meta::{VisualMeta, DEFAULT_VERSION};
use iced::widget::text_editor;
use chrono::Utc;
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
    /// Synchronization message processed by [`SyncEngine`].
    Sync(SyncMessage),
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
                let content = state.code_editor.text().to_string();
                handle_sync_message(state, SyncMessage::TextChanged(content, state.code_lang));
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
                        let meta = block_to_meta(block);
                        handle_sync_message(state, SyncMessage::VisualChanged(meta));
                    }
                }
                CanvasMessage::Dropped { position } => {
                    if let Some(Dragging::Palette(mut info)) = state.dragging.take() {
                        info.x = position.x as f64;
                        info.y = position.y as f64;
                        let meta = block_to_meta(&info);
                        state.blocks.push(info);
                        handle_sync_message(state, SyncMessage::VisualChanged(meta));
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
            MainMessage::Sync(msg) => {
                handle_sync_message(state, msg);
            }
            MainMessage::ShowConflict => {
                show_conflict_dialog(state);
            }
            MainMessage::ResolveConflict(id, Some(option)) => {
                state.sync_engine.apply_resolution(&id, option);
                update_sync_indicators(state);
                show_conflict_dialog(state);
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

/// Initialize synchronization engine with current editor state.
pub fn start_sync_engine(state: &mut MainUI) {
    let content = state.code_editor.text().to_string();
    handle_sync_message(state, SyncMessage::TextChanged(content, state.code_lang));
}

/// Process a [`SyncMessage`] through the engine and refresh indicators.
fn handle_sync_message(state: &mut MainUI, msg: SyncMessage) {
    if let Some((code, metas, diag)) = state.sync_engine.handle(msg) {
        let content = code.to_string();
        state.code_editor = text_editor::Content::with_text(&content);

        state.blocks = metas
            .iter()
            .map(|m| BlockInfo {
                visual_id: m.id.clone(),
                node_id: None,
                kind: String::new(),
                translations: m.translations.clone(),
                range: (0, 0),
                anchors: Vec::new(),
                x: m.x,
                y: m.y,
                ports: Vec::new(),
                ai: None,
                tags: m.tags.clone(),
                links: m.links.clone(),
            })
            .collect();

        state.connections = {
            let mut conns = Vec::new();
            for (i, block) in state.blocks.iter().enumerate() {
                for link in &block.links {
                    if let Some(j) = state.blocks.iter().position(|b| &b.visual_id == link) {
                        conns.push(Connection {
                            from: (i, 0),
                            to: (j, 0),
                            data_type: DataType::Any,
                        });
                    }
                }
            }
            conns
        };

        state.diagnostics = diag.clone();
    }
    update_sync_indicators(state);
}

/// Update stored conflicts after synchronization.
fn update_sync_indicators(state: &mut MainUI) {
    state.conflicts = state.sync_engine.last_conflicts().to_vec();
}

/// Display the next conflict in a dialog if any.
fn show_conflict_dialog(state: &mut MainUI) {
    state.active_conflict = state.conflicts.first().cloned();
}

fn block_to_meta(block: &BlockInfo) -> VisualMeta {
    VisualMeta {
        version: DEFAULT_VERSION,
        id: block.visual_id.clone(),
        x: block.x,
        y: block.y,
        tags: block.tags.clone(),
        links: block.links.clone(),
        anchors: Vec::new(),
        tests: Vec::new(),
        extends: None,
        origin: None,
        translations: block.translations.clone(),
        ai: None,
        extras: None,
        updated_at: Utc::now(),
    }
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

    #[test]
    fn handle_sync_updates_visual_state() {
        let mut ui = MainUI::default();
        let meta_b = VisualMeta {
            version: DEFAULT_VERSION,
            id: "b".into(),
            x: 10.0,
            y: 20.0,
            tags: Vec::new(),
            links: Vec::new(),
            anchors: Vec::new(),
            tests: Vec::new(),
            extends: None,
            origin: None,
            translations: std::collections::HashMap::new(),
            ai: None,
            extras: None,
            updated_at: Utc::now(),
        };
        handle_sync_message(&mut ui, SyncMessage::VisualChanged(meta_b));
        let meta_a = VisualMeta {
            version: DEFAULT_VERSION,
            id: "a".into(),
            x: 0.0,
            y: 0.0,
            tags: Vec::new(),
            links: vec!["b".into()],
            anchors: Vec::new(),
            tests: Vec::new(),
            extends: None,
            origin: None,
            translations: std::collections::HashMap::new(),
            ai: None,
            extras: None,
            updated_at: Utc::now(),
        };
        handle_sync_message(&mut ui, SyncMessage::VisualChanged(meta_a));

        let idx_a = ui.blocks.iter().position(|b| b.visual_id == "a").unwrap();
        let idx_b = ui.blocks.iter().position(|b| b.visual_id == "b").unwrap();
        assert!(ui
            .connections
            .iter()
            .any(|c| c.from.0 == idx_a && c.to.0 == idx_b));
    }
}
