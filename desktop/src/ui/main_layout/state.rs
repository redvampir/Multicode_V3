use super::view::{self, ModeView};
use super::update::{start_sync_engine, MainMessage};
use crate::app::ViewMode;
use crate::sync::{SyncConflict, SyncEngine, SyncSettings};
use crate::visual::connections::Connection;
use crate::visual::translations::Language;
use iced::widget::text_editor;
use iced::Element;
use multicode_core::parser::Lang;
use multicode_core::BlockInfo;

/// Main user interface state containing current view mode and editor states.
pub struct MainUI {
    /// Currently active view mode.
    pub view_mode: ViewMode,
    /// Internal state of the text editor.
    pub code_editor: text_editor::Content,
    /// Available blocks in the palette.
    pub palette: Vec<BlockInfo>,
    /// Blocks placed on the canvas.
    pub blocks: Vec<BlockInfo>,
    /// Connections between blocks.
    pub connections: Vec<Connection>,
    /// Currently dragged block (from the palette).
    pub dragging: Option<Dragging>,
    /// Current interface language for visual components.
    pub language: Language,
    /// Whether the block palette is visible.
    pub show_palette: bool,
    /// Registered view mode renderers allowing easy extension.
    pub view_modes: Vec<Box<dyn ModeView>>,
    /// Engine keeping text and visual representations in sync.
    pub sync_engine: SyncEngine,
    /// Conflicts detected during synchronization.
    pub conflicts: Vec<SyncConflict>,
    /// Currently visible conflict dialog.
    pub active_conflict: Option<SyncConflict>,
}

#[derive(Clone)]
pub enum Dragging {
    Palette(BlockInfo),
}

impl Default for MainUI {
    fn default() -> Self {
        let mut ui = Self {
            view_mode: ViewMode::Code,
            code_editor: text_editor::Content::new(),
            palette: load_palette(),
            blocks: Vec::new(),
            connections: Vec::new(),
            dragging: None,
            language: Language::default(),
            show_palette: true,
            view_modes: view::default_modes(),
            sync_engine: SyncEngine::new(Lang::Rust, SyncSettings::default()),
            conflicts: Vec::new(),
            active_conflict: None,
        };
        start_sync_engine(&mut ui);
        ui
    }
}

impl MainUI {
    /// Delegate message handling to the update module.
    pub fn update(&mut self, msg: MainMessage) {
        super::update::update(self, msg);
    }

    /// Delegate view rendering to the view module.
    pub fn view(&self) -> Element<MainMessage> {
        super::view::view(self)
    }
}

fn load_palette_with_lang(src: &str, lang: &str) -> Vec<BlockInfo> {
    match multicode_core::blocks::parse_blocks(src.to_string(), lang.into()) {
        Some(blocks) => blocks,
        None => {
            eprintln!("не удалось разобрать исходный код палитры");
            Vec::new()
        }
    }
}

fn load_palette_from(src: &str) -> Vec<BlockInfo> {
    load_palette_with_lang(src, "rust")
}

fn load_palette() -> Vec<BlockInfo> {
    let src = r#"
fn add(a: i32, b: i32) -> i32 { a + b }
fn mul(a: i32, b: i32) -> i32 { a * b }
"#;
    load_palette_from(src)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_palette_handles_corrupted_source() {
        let bad_src = "foo bar";
        let palette = super::load_palette_with_lang(bad_src, "invalid");
        assert!(palette.is_empty());
    }

    #[test]
    fn default_registers_view_modes() {
        let ui = MainUI::default();
        assert!(!ui.view_modes.is_empty());
    }
}
