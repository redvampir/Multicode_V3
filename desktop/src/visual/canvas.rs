use iced::widget::canvas::{self, Event, Frame, Geometry, Path, Program, Stroke, Text};
use iced::{mouse, Point, Rectangle, Renderer, Theme, Vector};

use multicode_core::BlockInfo;
use crate::visual::translations::{Language, translate_kind};

pub const BLOCK_WIDTH: f32 = 120.0;
pub const BLOCK_HEIGHT: f32 = 40.0;

#[derive(Debug, Clone)]
pub enum CanvasMessage {
    Pan { delta: Vector },
    Zoom { factor: f32, center: Point },
    BlockSelected(Option<usize>),
    BlockDragged { index: usize, position: Point },
}

pub struct VisualCanvas<'a> {
    blocks: &'a [BlockInfo],
    language: Language,
}

pub struct State {
    offset: Vector,
    scale: f32,
    selected: Option<usize>,
    drag: Option<Drag>,
    panning: bool,
    last_cursor: Point,
}

#[derive(Debug, Clone)]
struct Drag {
    index: usize,
    grab: Vector,
}

impl Default for State {
    fn default() -> Self {
        Self {
            offset: Vector::new(0.0, 0.0),
            scale: 1.0,
            selected: None,
            drag: None,
            panning: false,
            last_cursor: Point::ORIGIN,
        }
    }
}

impl<'a> VisualCanvas<'a> {
    pub fn new(blocks: &'a [BlockInfo], language: Language) -> Self {
        Self { blocks, language }
    }
}

fn contains(block: &BlockInfo, pos: Point) -> bool {
    pos.x >= block.x as f32
        && pos.x <= block.x as f32 + BLOCK_WIDTH
        && pos.y >= block.y as f32
        && pos.y <= block.y as f32 + BLOCK_HEIGHT
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
        match event {
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
        let mut frame = Frame::new(renderer, bounds.size());

        frame.translate(state.offset);
        frame.scale(state.scale);

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
