use crate::app::ViewMode;
use crate::visual::canvas::{CanvasMessage, VisualCanvas};
use crate::visual::connections::Connection;
use crate::visual::translations::Language;
use iced::widget::canvas::Canvas;
use iced::widget::{button, column, row, scrollable, text, text_editor};
use iced::{Element, Length, Sandbox};
use multicode_core::{blocks::parse_blocks, BlockInfo};

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
}

#[derive(Clone)]
pub enum Dragging {
    Palette(BlockInfo),
}

impl Default for MainUI {
    fn default() -> Self {
        Self {
            view_mode: ViewMode::Code,
            code_editor: text_editor::Content::new(),
            palette: load_palette(),
            blocks: Vec::new(),
            connections: Vec::new(),
            dragging: None,
            language: Language::default(),
            show_palette: true,
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
    /// Start dragging a block from the palette.
    StartPaletteDrag(usize),
    /// Message originating from the visual editor canvas.
    CanvasEvent(CanvasMessage),
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
            MainMessage::StartPaletteDrag(i) => {
                if let Some(info) = self.palette.get(i).cloned() {
                    self.dragging = Some(Dragging::Palette(info));
                }
            }
            MainMessage::CanvasEvent(event) => match event {
                CanvasMessage::BlockDragged { index, position } => {
                    if let Some(block) = self.blocks.get_mut(index) {
                        block.x = position.x as f64;
                        block.y = position.y as f64;
                    }
                }
                CanvasMessage::Dropped { position } => {
                    if let Some(Dragging::Palette(mut info)) = self.dragging.take() {
                        info.x = position.x as f64;
                        info.y = position.y as f64;
                        self.blocks.push(info);
                    }
                }
                CanvasMessage::ConnectionCreated(conn) => {
                    if !self.connections.contains(&conn) {
                        self.connections.push(conn);
                    }
                }
                CanvasMessage::TogglePalette => {
                    self.show_palette = !self.show_palette;
                }
                _ => {}
            },
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

    /// Create a view for the visual editor.
    fn create_visual_editor_view(&self) -> Element<MainMessage> {
        let canvas_widget = Canvas::new(VisualCanvas::new(
            &self.blocks,
            &self.connections,
            self.language,
        ))
        .width(Length::Fill)
        .height(Length::Fill);
        let canvas: Element<CanvasMessage> = canvas_widget.into();
        let canvas = canvas.map(MainMessage::CanvasEvent);
        if self.show_palette {
            let palette_column =
                self.palette
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

    /// Create a split view combining text and visual editors (placeholder).
    fn create_split_view(&self) -> Element<MainMessage> {
        row![
            self.create_text_editor_view(),
            self.create_visual_editor_view()
        ]
        .into()
    }
}

fn load_palette_with_lang(src: &str, lang: &str) -> Vec<BlockInfo> {
    match parse_blocks(src.to_string(), lang.into()) {
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
        let palette = load_palette_with_lang(bad_src, "invalid");
        assert!(palette.is_empty());
    }
}

impl Sandbox for MainUI {
    type Message = MainMessage;

    fn new() -> Self {
        Self::default()
    }

    fn title(&self) -> String {
        String::from("Multicode")
    }

    fn update(&mut self, message: Self::Message) {
        MainUI::update(self, message);
    }

    fn view(&self) -> Element<Self::Message> {
        MainUI::view(self)
    }
}
