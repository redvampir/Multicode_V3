use iced::widget::{button, column, row, scrollable, text};
use iced::widget::canvas::{self, Canvas, Event, Frame, Geometry, Path, Program, Stroke, Text};
use iced::{executor, mouse, Application, Command, Element, Length, Point, Rectangle, Renderer, Settings, Subscription, Theme};
use multicode_core::{blocks::parse_blocks, BlockInfo};

const BLOCK_WIDTH: f32 = 120.0;
const BLOCK_HEIGHT: f32 = 40.0;

#[derive(Debug, Clone)]
enum Message {
    StartPaletteDrag(usize),
    CanvasEvent(CanvasMessage),
}

#[derive(Debug, Clone)]
enum CanvasMessage {
    CursorMoved(Point),
    LeftPressed { index: Option<usize>, position: Point },
    LeftReleased { position: Point },
    RightPressed { index: Option<usize>, position: Point },
    RightReleased { index: Option<usize>, position: Point },
}

#[derive(Clone)]
struct PlacedBlock {
    info: BlockInfo,
}

enum Dragging {
    Palette(BlockInfo),
    Block { index: usize, offset: Point },
    Link { from: usize },
}

struct EditorApp {
    palette: Vec<BlockInfo>,
    blocks: Vec<PlacedBlock>,
    dragging: Option<Dragging>,
}

impl Application for EditorApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                palette: load_palette(),
                blocks: Vec::new(),
                dragging: None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Visual Editor")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::StartPaletteDrag(i) => {
                if let Some(info) = self.palette.get(i).cloned() {
                    self.dragging = Some(Dragging::Palette(info));
                }
            }
            Message::CanvasEvent(event) => match event {
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
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let palette_column = self
            .palette
            .iter()
            .enumerate()
            .fold(column!().spacing(5), |col, (i, b)| {
                col.push(button(text(&b.kind)).on_press(Message::StartPaletteDrag(i)))
            });
        let palette = scrollable(palette_column)
            .width(Length::Fixed(150.0))
            .height(Length::Fill);
        let canvas = Canvas::new(EditorCanvas { blocks: &self.blocks })
            .width(Length::Fill)
            .height(Length::Fill);
        row![palette, canvas].into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
}

struct EditorCanvas<'a> {
    blocks: &'a [PlacedBlock],
}

impl<'a> Program<Message> for EditorCanvas<'a> {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    return (
                        canvas::event::Status::Captured,
                        Some(Message::CanvasEvent(CanvasMessage::CursorMoved(pos))),
                    );
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(button)) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    let index = self.blocks.iter().position(|b| contains(b, pos));
                    let msg = match button {
                        mouse::Button::Left => {
                            Message::CanvasEvent(CanvasMessage::LeftPressed { index, position: pos })
                        }
                        mouse::Button::Right => {
                            Message::CanvasEvent(CanvasMessage::RightPressed { index, position: pos })
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
                        mouse::Button::Left => {
                            Message::CanvasEvent(CanvasMessage::LeftReleased { position: pos })
                        }
                        mouse::Button::Right => {
                            Message::CanvasEvent(CanvasMessage::RightReleased { index, position: pos })
                        }
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
            frame.fill_text(Text {
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

fn load_palette() -> Vec<BlockInfo> {
    let src = r#"
fn add(a: i32, b: i32) -> i32 { a + b }
fn mul(a: i32, b: i32) -> i32 { a * b }
"#;
    parse_blocks(src.to_string(), "rust".into()).unwrap_or_default()
}

pub fn main() -> iced::Result {
    EditorApp::run(Settings::default())
}
