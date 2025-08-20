use iced::advanced::text::highlighter::{self, Highlighter};
use iced::widget::{column, container, row, scrollable, text, text_editor};
use iced::{Color, Element, Length};

use crate::app::events::Message;

#[derive(Debug, Clone)]
pub struct DiffView {
    pub left: text_editor::Content,
    pub right: text_editor::Content,
    pub left_diff: Vec<usize>,
    pub right_diff: Vec<usize>,
    left_scroll: scrollable::Id,
    right_scroll: scrollable::Id,
}

impl DiffView {
    pub fn new(left: String, right: String) -> Self {
        let left_lines: Vec<&str> = left.lines().collect();
        let right_lines: Vec<&str> = right.lines().collect();
        let mut left_diff = Vec::new();
        let mut right_diff = Vec::new();
        let max = left_lines.len().max(right_lines.len());
        for i in 0..max {
            let l = left_lines.get(i);
            let r = right_lines.get(i);
            if l != r {
                if i < left_lines.len() {
                    left_diff.push(i);
                }
                if i < right_lines.len() {
                    right_diff.push(i);
                }
            }
        }
        let left_scroll = scrollable::Id::unique();
        let right_scroll = scrollable::Id::unique();
        scrollable::link(left_scroll.clone(), right_scroll.clone());

        Self {
            left: text_editor::Content::with_text(left),
            right: text_editor::Content::with_text(right),
            left_diff,
            right_diff,
            left_scroll,
            right_scroll,
        }
    }

    pub fn view(&self) -> Element<Message> {
        let left_editor =
            text_editor(&self.left).highlight::<LineHighlighter>(self.left_diff.clone(), |c, _| {
                highlighter::Format {
                    color: Some(*c),
                    font: None,
                }
            });
        let right_editor = text_editor(&self.right).highlight::<LineHighlighter>(
            self.right_diff.clone(),
            |c, _| highlighter::Format {
                color: Some(*c),
                font: None,
            },
        );
        let left_lines = column(
            (1..=self.left.line_count())
                .map(|i| text(i.to_string()).into())
                .collect::<Vec<Element<Message>>>(),
        );

        let right_lines = column(
            (1..=self.right.line_count())
                .map(|i| text(i.to_string()).into())
                .collect::<Vec<Element<Message>>>(),
        );

        let left_view = scrollable(
            row![
                container(left_lines).width(Length::Shrink),
                left_editor.height(Length::Shrink)
            ]
            .spacing(5),
        )
        .id(self.left_scroll.clone())
        .width(Length::FillPortion(1));

        let right_view = scrollable(
            row![
                container(right_lines).width(Length::Shrink),
                right_editor.height(Length::Shrink)
            ]
            .spacing(5),
        )
        .id(self.right_scroll.clone())
        .width(Length::FillPortion(1));

        row![left_view, right_view].spacing(10).into()
    }
}

#[derive(Debug, Clone)]
struct LineHighlighter {
    lines: Vec<usize>,
    current: usize,
}

impl Highlighter for LineHighlighter {
    type Settings = Vec<usize>;
    type Highlight = Color;
    type Iterator<'a> = std::vec::IntoIter<(std::ops::Range<usize>, Color)>;

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
            res.push((0..line.len(), Color::from_rgb(1.0, 0.0, 0.0)));
        }
        self.current += 1;
        res.into_iter()
    }

    fn current_line(&self) -> usize {
        self.current
    }
}
