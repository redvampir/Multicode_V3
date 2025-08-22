use crate::app::ViewMode;
use iced::widget::{
    button, column, row, scrollable, text, text_editor,
    canvas::{self, Canvas, Event, Frame, Geometry, Path, Program, Stroke, Text as CanvasText},
};
use iced::{
    executor, mouse, Application, Command, Element, Length, Point, Rectangle, Renderer, Subscription,
    Theme,
};
use multicode_core::{blocks::parse_blocks, BlockInfo};

/// Main user interface state containing current view mode and editor states.
pub struct MainUI {
    /// Currently active view mode.
    pub view_mode: ViewMode,
    /// Internal state of the text editor.
    pub code_editor: text_editor::Content,
    /// Available blocks for the palette.
    pub palette: Vec<BlockInfo>,
    /// Blocks placed on the canvas.
    pub blocks: Vec<PlacedBlock>,
    /// Currently active dragging operation.
    dragging: Option<Dragging>,
}

impl Default for MainUI {
    fn default() -> Self {
        Self {
            view_mode: ViewMode::Code,
            code_editor: text_editor::Content::new(),
            palette: load_palette(),
            blocks: Vec::new(),
            dragging: None,
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

#[derive(Debug, Clone)]
pub enum CanvasMessage {
    CursorMoved(Point),
    LeftPressed { index: Option<usize>, position: Point },
    LeftReleased { position: Point },
    RightPressed { index: Option<usize>, position: Point },
    RightReleased { index: Option<usize>, position: Point },
}

#[derive(Clone)]
pub struct PlacedBlock {
    info: BlockInfo,
}

enum Dragging {
    Palette(BlockInfo),
    Block { index: usize, offset: Point },
    Link { from: usize },
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
                CanvasMessage::CursorMoved(pos) => {
                    if let Some(Dragging::Block { index, offset }) = &mut self.dragging {
                        if let Some(block) = self.blocks.get_mut(*index) {
                            block.info.x = (pos.x - offset.x) as f64;
                            block.info.y = (pos.y - offset.y) as f64;
                        }
                    }
                }
                CanvasMessage::LeftPressed { index, position } => {
                    if let Some(i) = index {
                        if let Some(block) = self.blocks.get(i) {
                            let offset = Point::new(
                                position.x - block.info.x as f32,
                                position.y - block.info.y as f32,
                            );
                            self.dragging = Some(Dragging::Block { index: i, offset });
                        }
                    }
                }
                CanvasMessage::LeftReleased { position } => {
                    match self.dragging.take() {
                        Some(Dragging::Palette(mut info)) => {
                            info.x = position.x as f64;
                            info.y = position.y as f64;
                            self.blocks.push(PlacedBlock { info });
                        }
                        Some(Dragging::Block { .. }) => {}
                        _ => {}
                    }
                }
                CanvasMessage::RightPressed { index: Some(i), .. } => {
                    self.dragging = Some(Dragging::Link { from: i });
                }
                CanvasMessage::RightPressed { index: None, .. } => {}
                CanvasMessage::RightReleased { index, .. } => {
                    if let (Some(Dragging::Link { from }), Some(to)) = (self.dragging.take(), index) {
                        if from != to {
                            let id = self.blocks[to].info.visual_id.clone();
                            let links = &mut self.blocks[from].info.links;
                            if !links.contains(&id) {
                                links.push(id);
                            }
                        }
                    } else {
                        self.dragging = None;
                    }
                }
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
        let palette_column = self
            .palette
            .iter()
            .enumerate()
            .fold(column!().spacing(5), |col, (i, b)| {
                col.push(button(text(&b.kind)).on_press(MainMessage::StartPaletteDrag(i)))
            });
        let palette = scrollable(palette_column)
            .width(Length::Fixed(150.0))
            .height(Length::Fill);
        let canvas = Canvas::new(EditorCanvas { blocks: &self.blocks })
            .width(Length::Fill)
            .height(Length::Fill);
        row![palette, canvas].into()
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

struct EditorCanvas<'a> {
    blocks: &'a [PlacedBlock],
}

impl<'a> Program<MainMessage> for EditorCanvas<'a> {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<MainMessage>) {
        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    return (
                        canvas::event::Status::Captured,
                        Some(MainMessage::CanvasEvent(CanvasMessage::CursorMoved(pos))),
                    );
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(button)) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    let index = self.blocks.iter().position(|b| contains(b, pos));
                    let msg = match button {
                        mouse::Button::Left => MainMessage::CanvasEvent(CanvasMessage::LeftPressed {
                            index,
                            position: pos,
                        }),
                        mouse::Button::Right => {
                            MainMessage::CanvasEvent(CanvasMessage::RightPressed { index, position: pos })
                        }
                        _ => return (canvas::event::Status::Ignored, None),
                    };
                    return (canvas::event::Status::Captured, Some(msg));
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(button)) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    let index = self.blocks.iter().position(|b| contains(b, pos));
                    let msg = match button {
                        mouse::Button::Left => MainMessage::CanvasEvent(CanvasMessage::LeftReleased { position: pos }),
                        mouse::Button::Right => MainMessage::CanvasEvent(CanvasMessage::RightReleased { index, position: pos }),
                        _ => return (canvas::event::Status::Ignored, None),
                    };
                    return (canvas::event::Status::Captured, Some(msg));
                }
            }
            _ => {}
        }
        (canvas::event::Status::Ignored, None)
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        for block in self.blocks.iter() {
            let rect = Path::rectangle(
                Point::new(block.info.x as f32, block.info.y as f32),
                iced::Size::new(BLOCK_WIDTH, BLOCK_HEIGHT),
            );
            frame.fill(&rect, iced::Color::from_rgb(0.3, 0.3, 0.7));
            frame.stroke(&rect, Stroke::default());
            frame.fill_text(CanvasText {
                content: block.kind_name(),
                position: Point::new(block.info.x as f32 + 5.0, block.info.y as f32 + 20.0),
                color: iced::Color::BLACK,
                ..Default::default()
            });
        }

        for block in self.blocks.iter() {
            let start = Point::new(
                block.info.x as f32 + BLOCK_WIDTH / 2.0,
                block.info.y as f32 + BLOCK_HEIGHT / 2.0,
            );
            for link in &block.info.links {
                if let Some(target) = self.blocks.iter().find(|b| &b.info.visual_id == link) {
                    let end = Point::new(
                        target.info.x as f32 + BLOCK_WIDTH / 2.0,
                        target.info.y as f32 + BLOCK_HEIGHT / 2.0,
                    );
                    frame.stroke(&Path::line(start, end), Stroke::default().with_width(2.0));
                }
            }
        }

        vec![frame.into_geometry()]
    }
}

impl PlacedBlock {
    fn kind_name(&self) -> String {
        self.info.kind.clone()
    }
}

fn contains(block: &PlacedBlock, pos: Point) -> bool {
    pos.x >= block.info.x as f32
        && pos.x <= block.info.x as f32 + BLOCK_WIDTH
        && pos.y >= block.info.y as f32
        && pos.y <= block.info.y as f32 + BLOCK_HEIGHT
}

const BLOCK_WIDTH: f32 = 120.0;
const BLOCK_HEIGHT: f32 = 40.0;

fn load_palette() -> Vec<BlockInfo> {
    let src = r#"\
fn add(a: i32, b: i32) -> i32 { a + b }
fn mul(a: i32, b: i32) -> i32 { a * b }
"#;
    parse_blocks(src.to_string(), "rust".into()).unwrap_or_default()
}

impl Application for MainUI {
    type Executor = executor::Default;
    type Message = MainMessage;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("Multicode")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        MainUI::update(self, message);
        Command::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::none()
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }

    fn view(&self) -> Element<Self::Message> {
        MainUI::view(self)
    }
}
