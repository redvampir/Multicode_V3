use iced::widget::canvas::{Frame, Path, Stroke};
use iced::{Color, Point, Vector};
use multicode_core::BlockInfo;

use crate::visual::connections::{Connection, DataType};

pub const PORT_RADIUS: f32 = 5.0;
const ARROW_LENGTH: f32 = 10.0;
const ARROW_WIDTH: f32 = 6.0;

#[derive(Debug, Clone)]
pub struct ConnectionDrag {
    pub from_block: usize,
    pub from_port: usize,
    pub start: Point,
    pub current: Point,
    pub hover: Option<(usize, usize)>,
    pub data_type: DataType,
}

#[derive(Clone)]
pub struct PreparedConnection {
    pub start: Point,
    pub end: Point,
    pub color: Color,
}

pub fn prepare_connections(
    blocks: &[BlockInfo],
    connections: &[Connection],
) -> Vec<PreparedConnection> {
    let mut prepared = Vec::new();
    for c in connections {
        if let (Some(from_block), Some(to_block)) = (blocks.get(c.from.0), blocks.get(c.to.0)) {
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
                    DataType::Any => Color::from_rgb(0.5, 0.5, 0.5),
                };
                prepared.push(PreparedConnection { start, end, color });
            }
        }
    }
    prepared
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

pub fn draw_connections(frame: &mut Frame, connections: &[PreparedConnection], scale: f32) {
    for connection in connections.iter() {
        let path = Path::line(connection.start, connection.end);
        let stroke = Stroke::default()
            .with_color(connection.color)
            .with_width(2.0);
        frame.stroke(&path, stroke);
        draw_arrow(
            frame,
            connection.start,
            connection.end,
            connection.color,
            scale,
        );
    }
}

pub fn draw_drag(frame: &mut Frame, conn: &ConnectionDrag, blocks: &[BlockInfo], scale: f32) {
    let path = Path::line(conn.start, conn.current);
    let stroke = Stroke::default()
        .with_color(Color::from_rgb(0.0, 0.0, 0.8))
        .with_width(2.0);
    frame.stroke(&path, stroke);
    draw_arrow(
        frame,
        conn.start,
        conn.current,
        Color::from_rgb(0.0, 0.0, 0.8),
        scale,
    );
    if let Some((b, p)) = conn.hover {
        if let Some(block) = blocks.get(b) {
            if let Some(port) = block.ports.get(p) {
                let point = Point::new((block.x + port.x) as f32, (block.y + port.y) as f32);
                let circle = Path::circle(point, PORT_RADIUS);
                frame.fill(&circle, Color::from_rgba(0.0, 1.0, 0.0, 0.5));
            }
        }
    }
}
