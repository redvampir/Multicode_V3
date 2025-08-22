use iced::widget::canvas::{self, Event, Frame, Geometry, Path, Program, Stroke, Text};
use iced::{
    keyboard::{self, key},
    mouse, Color, Point, Rectangle, Renderer, Theme, Vector,
};
use std::cell::RefCell;

use crate::visual::translations::{translate_kind, Language};
use multicode_core::BlockInfo;

pub const BLOCK_WIDTH: f32 = 120.0;
pub const BLOCK_HEIGHT: f32 = 40.0;
const PORT_RADIUS: f32 = 5.0;
const ARROW_LENGTH: f32 = 10.0;
const ARROW_WIDTH: f32 = 6.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    Number,
    Boolean,
    Text,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Connection {
    pub from: (usize, usize),
    pub to: (usize, usize),
    pub data_type: DataType,
}

#[derive(Debug, Clone)]
pub enum CanvasMessage {
    Pan { delta: Vector },
    Zoom { factor: f32, center: Point },
    BlockSelected(Option<usize>),
    BlockDragged { index: usize, position: Point },
    Dropped { position: Point },
    TogglePalette,
    ConnectionCreated(Connection),
}

pub struct VisualCanvas<'a> {
    blocks: &'a [BlockInfo],
    connections: &'a [Connection],
    language: Language,
}

pub struct State {
    offset: Vector,
    scale: f32,
    selected: Option<usize>,
    drag: Option<Drag>,
    connection: Option<ConnectionDrag>,
    panning: bool,
    last_cursor: Point,
    connections: RefCell<Vec<PreparedConnection>>,
    last_blocks: RefCell<Vec<(f64, f64)>>,
    last_connections: RefCell<Vec<Connection>>,
}

#[derive(Debug, Clone)]
struct Drag {
    index: usize,
    grab: Vector,
}

#[derive(Debug, Clone)]
struct ConnectionDrag {
    from_block: usize,
    from_port: usize,
    start: Point,
    current: Point,
    hover: Option<(usize, usize)>,
    data_type: DataType,
}

#[derive(Clone)]
struct PreparedConnection {
    start: Point,
    end: Point,
    color: Color,
}

impl Default for State {
    fn default() -> Self {
        Self {
            offset: Vector::new(0.0, 0.0),
            scale: 1.0,
            selected: None,
            drag: None,
            connection: None,
            panning: false,
            last_cursor: Point::ORIGIN,
            connections: RefCell::new(Vec::new()),
            last_blocks: RefCell::new(Vec::new()),
            last_connections: RefCell::new(Vec::new()),
        }
    }
}

impl<'a> VisualCanvas<'a> {
    pub fn new(blocks: &'a [BlockInfo], connections: &'a [Connection], language: Language) -> Self {
        Self {
            blocks,
            connections,
            language,
        }
    }
}

fn contains(block: &BlockInfo, pos: Point) -> bool {
    pos.x >= block.x as f32
        && pos.x <= block.x as f32 + BLOCK_WIDTH
        && pos.y >= block.y as f32
        && pos.y <= block.y as f32 + BLOCK_HEIGHT
}

fn find_port(blocks: &[BlockInfo], pos: Point, output: bool) -> Option<(usize, usize, Point)> {
    for (bi, block) in blocks.iter().enumerate() {
        for (pi, port) in block.ports.iter().enumerate() {
            let port_pos = Point::new((block.x + port.x) as f32, (block.y + port.y) as f32);
            let dx = pos.x - port_pos.x;
            let dy = pos.y - port_pos.y;
            if dx * dx + dy * dy <= PORT_RADIUS * PORT_RADIUS {
                let is_output = port.x >= (BLOCK_WIDTH - PORT_RADIUS * 2.0) as f64;
                if is_output == output {
                    return Some((bi, pi, port_pos));
                }
            }
        }
    }
    None
}

fn draw_arrow(frame: &mut Frame, start: Point, end: Point, color: Color, scale: f32) {
    let dir = Vector::new(end.x - start.x, end.y - start.y);
    let length = (dir.x * dir.x + dir.y * dir.y).sqrt();
    if length == 0.0 {
        return;
    }
    let norm = Vector::new(dir.x / length, dir.y / length);
    let perp = Vector::new(-norm.y, norm.x);
    let arrow_length = ARROW_LENGTH / scale;
    let arrow_width = ARROW_WIDTH / scale;
    let base = Point::new(end.x - norm.x * arrow_length, end.y - norm.y * arrow_length);
    let left = Point::new(
        base.x + perp.x * (arrow_width / 2.0),
        base.y + perp.y * (arrow_width / 2.0),
    );
    let right = Point::new(
        base.x - perp.x * (arrow_width / 2.0),
        base.y - perp.y * (arrow_width / 2.0),
    );
    let triangle = Path::new(|p| {
        p.move_to(end);
        p.line_to(left);
        p.line_to(right);
        p.close();
    });
    frame.fill(&triangle, color);
}

impl State {
    fn update_connections(&self, blocks: &[BlockInfo], connections: &[Connection]) {
        let current_blocks: Vec<(f64, f64)> = blocks.iter().map(|b| (b.x, b.y)).collect();
        let mut last_blocks = self.last_blocks.borrow_mut();
        let mut last_connections = self.last_connections.borrow_mut();
        if *last_blocks != current_blocks || *last_connections != connections {
            let mut prepared = Vec::new();
            for c in connections {
                if let (Some(from_block), Some(to_block)) =
                    (blocks.get(c.from.0), blocks.get(c.to.0))
                {
                    if let (Some(from_port), Some(to_port)) =
                        (from_block.ports.get(c.from.1), to_block.ports.get(c.to.1))
                    {
                        let start = Point::new(
                            (from_block.x + from_port.x) as f32,
                            (from_block.y + from_port.y) as f32,
                        );
                        let end = Point::new(
                            (to_block.x + to_port.x) as f32,
                            (to_block.y + to_port.y) as f32,
                        );
                        let color = match c.data_type {
                            DataType::Number => Color::from_rgb(0.0, 0.0, 0.8),
                            DataType::Boolean => Color::from_rgb(0.0, 0.6, 0.0),
                            DataType::Text => Color::from_rgb(1.0, 0.5, 0.0),
                        };
                        prepared.push(PreparedConnection { start, end, color });
                    }
                }
            }
            *self.connections.borrow_mut() = prepared;
            *last_blocks = current_blocks;
            *last_connections = connections.to_vec();
        }
    }
}

impl<'a> Program<CanvasMessage> for VisualCanvas<'a> {
    type State = State;

    fn update(
        &self,
        state: &mut Self::State,
        event: Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<CanvasMessage>) {
        state.update_connections(self.blocks, self.connections);
        match event {
            Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                if key == keyboard::Key::Named(key::Named::Space) {
                    return (
                        canvas::event::Status::Captured,
                        Some(CanvasMessage::TogglePalette),
                    );
                }
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    let y = match delta {
                        mouse::ScrollDelta::Lines { y, .. } => y,
                        mouse::ScrollDelta::Pixels { y, .. } => y / 120.0,
                    };
                    let factor = if y > 0.0 { 1.1 } else { 0.9 };
                    state.scale = (state.scale * factor).clamp(0.1, 4.0);
                    return (
                        canvas::event::Status::Captured,
                        Some(CanvasMessage::Zoom {
                            factor: state.scale,
                            center: pos,
                        }),
                    );
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(button)) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    match button {
                        mouse::Button::Left => {
                            let canvas_pos = Point::new(
                                (pos.x - state.offset.x) / state.scale,
                                (pos.y - state.offset.y) / state.scale,
                            );
                            if state.connection.is_none() {
                                if let Some((b, p, start)) =
                                    find_port(self.blocks, canvas_pos, true)
                                {
                                    state.connection = Some(ConnectionDrag {
                                        from_block: b,
                                        from_port: p,
                                        start,
                                        current: start,
                                        hover: None,
                                        data_type: DataType::Number,
                                    });
                                    return (canvas::event::Status::Captured, None);
                                }
                            }
                            if let Some((idx, block)) = self
                                .blocks
                                .iter()
                                .enumerate()
                                .find(|(_, b)| contains(b, canvas_pos))
                            {
                                state.selected = Some(idx);
                                let grab = Vector::new(
                                    canvas_pos.x - block.x as f32,
                                    canvas_pos.y - block.y as f32,
                                );
                                state.drag = Some(Drag { index: idx, grab });
                                return (
                                    canvas::event::Status::Captured,
                                    Some(CanvasMessage::BlockSelected(Some(idx))),
                                );
                            } else {
                                state.selected = None;
                                state.drag = None;
                                return (
                                    canvas::event::Status::Captured,
                                    Some(CanvasMessage::BlockSelected(None)),
                                );
                            }
                        }
                        mouse::Button::Right => {
                            state.panning = true;
                            state.last_cursor = pos;
                            return (canvas::event::Status::Captured, None);
                        }
                        _ => {}
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(button)) => match button {
                mouse::Button::Left => {
                    if let Some(conn) = state.connection.take() {
                        if let Some((to_block, to_port)) = conn.hover {
                            let connection = Connection {
                                from: (conn.from_block, conn.from_port),
                                to: (to_block, to_port),
                                data_type: conn.data_type,
                            };
                            return (
                                canvas::event::Status::Captured,
                                Some(CanvasMessage::ConnectionCreated(connection)),
                            );
                        } else {
                            return (canvas::event::Status::Captured, None);
                        }
                    }
                    if let Some(drag) = state.drag.take() {
                        if let Some(pos) = cursor.position_in(bounds) {
                            let canvas_pos = Point::new(
                                (pos.x - state.offset.x) / state.scale,
                                (pos.y - state.offset.y) / state.scale,
                            );
                            let new_pos =
                                Point::new(canvas_pos.x - drag.grab.x, canvas_pos.y - drag.grab.y);
                            return (
                                canvas::event::Status::Captured,
                                Some(CanvasMessage::BlockDragged {
                                    index: drag.index,
                                    position: new_pos,
                                }),
                            );
                        }
                    } else if let Some(pos) = cursor.position_in(bounds) {
                        let canvas_pos = Point::new(
                            (pos.x - state.offset.x) / state.scale,
                            (pos.y - state.offset.y) / state.scale,
                        );
                        return (
                            canvas::event::Status::Captured,
                            Some(CanvasMessage::Dropped {
                                position: canvas_pos,
                            }),
                        );
                    }
                }
                mouse::Button::Right => {
                    if state.panning {
                        state.panning = false;
                        return (canvas::event::Status::Captured, None);
                    }
                }
                _ => {}
            },
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    if state.panning {
                        let delta = pos - state.last_cursor;
                        state.last_cursor = pos;
                        state.offset = state.offset + delta;
                        return (
                            canvas::event::Status::Captured,
                            Some(CanvasMessage::Pan { delta }),
                        );
                    }
                    if let Some(conn) = state.connection.as_mut() {
                        let canvas_pos = Point::new(
                            (pos.x - state.offset.x) / state.scale,
                            (pos.y - state.offset.y) / state.scale,
                        );
                        conn.current = canvas_pos;
                        conn.hover =
                            find_port(self.blocks, canvas_pos, false).map(|(b, p, _)| (b, p));
                        return (canvas::event::Status::Captured, None);
                    }
                    if let Some(drag) = state.drag.as_ref() {
                        let canvas_pos = Point::new(
                            (pos.x - state.offset.x) / state.scale,
                            (pos.y - state.offset.y) / state.scale,
                        );
                        let new_pos =
                            Point::new(canvas_pos.x - drag.grab.x, canvas_pos.y - drag.grab.y);
                        return (
                            canvas::event::Status::Captured,
                            Some(CanvasMessage::BlockDragged {
                                index: drag.index,
                                position: new_pos,
                            }),
                        );
                    }
                }
            }
            _ => {}
        }
        (canvas::event::Status::Ignored, None)
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        state.update_connections(self.blocks, self.connections);

        let mut frame = Frame::new(renderer, bounds.size());

        frame.translate(state.offset);
        frame.scale(state.scale);

        let connections = state.connections.borrow();
        for connection in connections.iter() {
            let path = Path::line(connection.start, connection.end);
            let stroke = Stroke::default()
                .with_color(connection.color)
                .with_width(2.0);
            frame.stroke(&path, stroke);
            draw_arrow(
                &mut frame,
                connection.start,
                connection.end,
                connection.color,
                state.scale,
            );
        }
        drop(connections);

        if let Some(conn) = state.connection.as_ref() {
            let path = Path::line(conn.start, conn.current);
            let stroke = Stroke::default()
                .with_color(Color::from_rgb(0.0, 0.0, 0.8))
                .with_width(2.0);
            frame.stroke(&path, stroke);
            draw_arrow(
                &mut frame,
                conn.start,
                conn.current,
                Color::from_rgb(0.0, 0.0, 0.8),
                state.scale,
            );
            if let Some((b, p)) = conn.hover {
                if let Some(block) = self.blocks.get(b) {
                    if let Some(port) = block.ports.get(p) {
                        let point =
                            Point::new((block.x + port.x) as f32, (block.y + port.y) as f32);
                        let circle = Path::circle(point, PORT_RADIUS);
                        frame.fill(&circle, Color::from_rgba(0.0, 1.0, 0.0, 0.5));
                    }
                }
            }
        }

        for (i, block) in self.blocks.iter().enumerate() {
            let rect = Path::rectangle(
                Point::new(block.x as f32, block.y as f32),
                iced::Size::new(BLOCK_WIDTH, BLOCK_HEIGHT),
            );
            let color = if state.selected == Some(i) {
                iced::Color::from_rgb(0.8, 0.3, 0.3)
            } else {
                iced::Color::from_rgb(0.3, 0.3, 0.7)
            };
            frame.fill(&rect, color);
            frame.stroke(&rect, Stroke::default());
            let label = block
                .translations
                .get(self.language.code())
                .cloned()
                .or_else(|| translate_kind(&block.kind, self.language).map(|s| s.to_string()))
                .unwrap_or_else(|| block.kind.clone());
            frame.fill_text(Text {
                content: label,
                position: Point::new(block.x as f32 + 5.0, block.y as f32 + 20.0),
                color: iced::Color::BLACK,
                ..Default::default()
            });
        }

        vec![frame.into_geometry()]
    }
}
