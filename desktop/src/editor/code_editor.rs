use std::ops::Range;

use chrono::DateTime;
use iced::advanced::text::highlighter::{self, Highlighter};
use iced::widget::{
    button, column, container, row, scrollable, text, text_editor,
    tooltip::{self, Tooltip},
    Column,
};
use iced::{Alignment, Color, Element, Length};
use once_cell::sync::Lazy;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

use crate::app::events::Message;
use crate::app::MulticodeApp;

static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
pub static THEME_SET: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);

fn load_highlighting(
    extension: &str,
    theme: &str,
) -> (
    &'static syntect::parsing::SyntaxReference,
    &'static syntect::highlighting::Theme,
) {
    let syntax = SYNTAX_SET
        .find_syntax_by_extension(extension)
        .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());
    let theme = THEME_SET
        .themes
        .get(theme)
        .unwrap_or(&THEME_SET.themes["InspiredGitHub"]);
    (syntax, theme)
}

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

#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxSettings {
    pub extension: String,
    pub matches: Vec<(usize, Range<usize>)>,
    pub diagnostics: Vec<(usize, Range<usize>)>,
    pub theme: String,
    pub match_color: Color,
    pub diagnostic_color: Color,
}

pub struct SyntectHighlighter {
    settings: SyntaxSettings,
    highlighter: HighlightLines<'static>,
    current_line: usize,
}

impl Highlighter for SyntectHighlighter {
    type Settings = SyntaxSettings;
    type Highlight = Color;
    type Iterator<'a> = std::vec::IntoIter<(Range<usize>, Color)>;

    fn new(settings: &Self::Settings) -> Self {
        let (syntax, theme) = load_highlighting(&settings.extension, &settings.theme);
        Self {
            settings: settings.clone(),
            highlighter: HighlightLines::new(syntax, theme),
            current_line: 0,
        }
    }

    fn update(&mut self, new_settings: &Self::Settings) {
        let (syntax, theme) = load_highlighting(&new_settings.extension, &new_settings.theme);
        self.highlighter = HighlightLines::new(syntax, theme);
        self.settings = new_settings.clone();
        self.current_line = 0;
    }

    fn change_line(&mut self, line: usize) {
        self.current_line = line;
    }

    fn highlight_line(&mut self, line: &str) -> Self::Iterator<'_> {
        let mut res = Vec::new();
        if let Ok(ranges) = self.highlighter.highlight_line(line, &SYNTAX_SET) {
            let mut start = 0;
            for (style, text) in ranges {
                let len = text.len();
                let color = Color::from_rgb(
                    style.foreground.r as f32 / 255.0,
                    style.foreground.g as f32 / 255.0,
                    style.foreground.b as f32 / 255.0,
                );
                res.push((start..start + len, color));
                start += len;
            }
        }
        for (line_idx, range) in &self.settings.matches {
            if *line_idx == self.current_line {
                res.push((range.clone(), self.settings.match_color));
            }
        }
        for (line_idx, range) in &self.settings.diagnostics {
            if *line_idx == self.current_line {
                res.push((range.clone(), self.settings.diagnostic_color));
            }
        }
        self.current_line += 1;
        res.into_iter()
    }

    fn current_line(&self) -> usize {
        self.current_line
    }
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
            let settings = SyntaxSettings {
                extension: ext.clone(),
                matches: self.app.search_results().to_vec(),
                diagnostics: file
                    .diagnostics
                    .iter()
                    .map(|d| (d.line, d.range.clone()))
                    .collect(),
                theme: self.app.settings().syntect_theme.clone(),
                match_color: self.app.settings().match_color,
                diagnostic_color: self.app.settings().diagnostic_color,
            };
            let editor = text_editor(&file.editor)
                .highlight::<SyntectHighlighter>(settings, |c, _| highlighter::Format {
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

            let editor_column = column![editor_view];
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
