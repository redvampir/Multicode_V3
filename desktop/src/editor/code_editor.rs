use chrono::DateTime;
use iced::advanced::text::highlighter;
use iced::widget::{
    button, column, container, row, scrollable, text, text_editor,
    tooltip::{self, Tooltip},
    Column,
};
use iced::{theme, Alignment, Color, Element, Length};

use crate::app::events::Message;
use crate::app::MulticodeApp;

use super::syntax_highlighter::{SyntaxHighlighter, SyntaxSettings, SyntaxColors};

pub(super) fn markdown_preview(content: &str) -> Column<'static, Message> {
    use pulldown_cmark::{Event, HeadingLevel, Parser, Tag};

    let parser = Parser::new(content);
    let mut elements: Vec<Element<Message>> = Vec::new();
    let mut buf = String::new();
    let mut heading = None;

    for event in parser {
        match event {
            Event::Start(Tag::Heading(level, _, _)) => {
                buf.clear();
                heading = Some(level);
            }
            Event::End(Tag::Heading(_, _, _)) => {
                let size = match heading.unwrap_or(HeadingLevel::H1) {
                    HeadingLevel::H1 => 30.0,
                    HeadingLevel::H2 => 26.0,
                    HeadingLevel::H3 => 22.0,
                    _ => 20.0,
                };
                elements.push(text(buf.clone()).size(size).into());
                buf.clear();
                heading = None;
            }
            Event::Start(Tag::Paragraph) => buf.clear(),
            Event::End(Tag::Paragraph) => {
                elements.push(text(buf.clone()).into());
                buf.clear();
            }
            Event::Start(Tag::Item) => buf.push_str("• "),
            Event::End(Tag::Item) => {
                elements.push(text(buf.clone()).into());
                buf.clear();
            }
            Event::Text(t) | Event::Code(t) => buf.push_str(&t),
            Event::SoftBreak | Event::HardBreak => buf.push('\n'),
            _ => {}
        }
    }

    if !buf.is_empty() {
        elements.push(text(buf).into());
    }

    column(elements).spacing(5)
}


pub struct CodeEditor<'a> {
    app: &'a MulticodeApp,
}

impl<'a> CodeEditor<'a> {
    pub fn new(app: &'a MulticodeApp) -> Self {
        Self { app }
    }

    pub fn view(self) -> Element<'a, Message> {
        if let Some(file) = self.app.current_file() {
            let ext = file
                .path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_string();
            let colors = SyntaxColors::for_theme(
                &self.app.settings().syntect_theme,
                self.app.settings().match_color,
                self.app.settings().diagnostic_color,
            );
            let settings = SyntaxSettings {
                extension: ext.clone(),
                matches: self.app.search_results().to_vec(),
                diagnostics: file
                    .diagnostics
                    .iter()
                    .map(|d| (d.line, d.range.clone()))
                    .collect(),
                theme: self.app.settings().syntect_theme.clone(),
                colors,
            };
            let editor = text_editor(&file.editor)
                .highlight::<SyntaxHighlighter>(settings, |c, _| highlighter::Format {
                    color: Some(*c),
                    font: None,
                })
                .on_action(Message::FileContentEdited);

            let editor_view: Element<'a, Message> = if self.app.settings().show_line_numbers {
                let lines = column(
                    (1..=file.editor.line_count())
                        .map(|i| {
                            let ln = text(i.to_string());
                            if let Some(info) = file.blame.get(&i) {
                                let tooltip_text =
                                    if let Some(dt) = DateTime::from_timestamp(info.time, 0) {
                                        format!("{} – {}", info.author, dt.format("%Y-%m-%d"))
                                    } else {
                                        format!("{} – unknown date", info.author)
                                    };
                                Tooltip::new(
                                    ln,
                                    text(tooltip_text),
                                    tooltip::Position::FollowCursor,
                                )
                                .into()
                            } else {
                                ln.into()
                            }
                        })
                        .collect::<Vec<Element<Message>>>(),
                );

                scrollable(
                    row![
                        container(lines).width(Length::Shrink),
                        editor.height(Length::Shrink)
                    ]
                    .spacing(5),
                )
                .height(Length::Fill)
                .into()
            } else {
                editor.height(Length::Fill).into()
            };

            let mut editor_column = column![editor_view];
            if let Some(ac) = self.app.autocomplete() {
                let items = ac.suggestions.iter().enumerate().fold(
                    column![]
                        .spacing(2),
                    |col, (i, s)| {
                        let t = if i == ac.selected {
                            text(&s.label)
                                .style(theme::Text::Color(Color::from_rgb(0.3, 0.3, 1.0)))
                        } else {
                            text(&s.label)
                        };
                        col.push(t)
                    },
                );
                let popup = container(items)
                    .padding(5)
                    .style(theme::Container::Box);
                editor_column = editor_column.push(popup);
            }
            if self.app.settings().show_markdown_preview && ext == "md" {
                let preview =
                    scrollable(markdown_preview(&file.content)).width(Length::FillPortion(1));
                let main = row![editor_column.width(Length::FillPortion(1)), preview].spacing(5);
                if self.app.show_meta_panel() {
                    row![
                        main.width(Length::FillPortion(3)),
                        self.app.meta_panel_component()
                    ]
                    .spacing(5)
                    .into()
                } else {
                    main.into()
                }
            } else {
                if self.app.show_meta_panel() {
                    row![
                        editor_column.width(Length::FillPortion(3)),
                        self.app.meta_panel_component()
                    ]
                    .spacing(5)
                    .into()
                } else {
                    editor_column.into()
                }
            }
        } else {
            container(
                column![
                    button("Открыть файл").on_press(Message::PickFile),
                    text("Файл можно перетащить в окно"),
                ]
                .spacing(5)
                .align_items(Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
        }
    }
}
