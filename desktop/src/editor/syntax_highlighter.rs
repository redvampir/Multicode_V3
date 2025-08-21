use std::ops::Range;

use iced::advanced::text::highlighter::Highlighter;
use iced::Color;
use std::collections::HashMap;
use once_cell::sync::Lazy;
use regex::Regex;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::SyntaxSet;

pub static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
pub static THEME_SET: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);

static META_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"@VISUAL_META").unwrap());

#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxColors {
    pub match_color: Color,
    pub diagnostic_color: Color,
    pub meta_color: Color,
}

impl SyntaxColors {
    pub fn for_theme(theme: &str, match_color: Color, diagnostic_color: Color) -> Self {
        let meta_color = theme_brightness(theme)
            .map(|dark| if dark { Color::from_rgb(1.0, 0.5, 0.0) } else { Color::from_rgb(0.5, 0.0, 0.5) })
            .unwrap_or(Color::from_rgb(0.5, 0.0, 0.5));
        Self { match_color, diagnostic_color, meta_color }
    }
}

fn theme_brightness(name: &str) -> Option<bool> {
    let theme: &Theme = THEME_SET.themes.get(name)?;
    let bg = theme.settings.background?;
    let luminance = 0.299 * bg.r as f32 + 0.587 * bg.g as f32 + 0.114 * bg.b as f32;
    Some(luminance < 128.0)
}

#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxSettings {
    pub extension: String,
    pub matches: Vec<(usize, Range<usize>)>,
    pub diagnostics: Vec<(usize, Range<usize>)>,
    pub theme: String,
    pub colors: SyntaxColors,
}

pub struct SyntaxHighlighter {
    settings: SyntaxSettings,
    highlighter: HighlightLines<'static>,
    current_line: usize,
    cache: HashMap<usize, Vec<(Range<usize>, Color)>>,
}

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

impl Highlighter for SyntaxHighlighter {
    type Settings = SyntaxSettings;
    type Highlight = Color;
    type Iterator<'a> = std::vec::IntoIter<(Range<usize>, Color)>;

    fn new(settings: &Self::Settings) -> Self {
        let (syntax, theme) = load_highlighting(&settings.extension, &settings.theme);
        Self {
            settings: settings.clone(),
            highlighter: HighlightLines::new(syntax, theme),
            current_line: 0,
            cache: HashMap::new(),
        }
    }

    fn update(&mut self, new_settings: &Self::Settings) {
        let (syntax, theme) = load_highlighting(&new_settings.extension, &new_settings.theme);
        self.highlighter = HighlightLines::new(syntax, theme);
        self.settings = new_settings.clone();
        self.current_line = 0;
        self.cache.clear();
    }

    fn change_line(&mut self, line: usize) {
        self.current_line = line;
    }

    fn highlight_line(&mut self, line: &str) -> Self::Iterator<'_> {
        let mut res = if let Some(cached) = self.cache.get(&self.current_line) {
            cached.clone()
        } else {
            let mut tmp = Vec::new();
            if let Ok(ranges) = self.highlighter.highlight_line(line, &SYNTAX_SET) {
                let mut start = 0;
                for (style, text) in ranges {
                    let len = text.len();
                    let color = Color::from_rgb(
                        style.foreground.r as f32 / 255.0,
                        style.foreground.g as f32 / 255.0,
                        style.foreground.b as f32 / 255.0,
                    );
                    tmp.push((start..start + len, color));
                    start += len;
                }
            }
            tmp
        };
        if let Some(pos) = META_REGEX.find(line).map(|m| m.start()) {
            res.push((pos..line.len(), self.settings.colors.meta_color));
        }
        for (line_idx, range) in &self.settings.matches {
            if *line_idx == self.current_line {
                res.push((range.clone(), self.settings.colors.match_color));
            }
        }
        for (line_idx, range) in &self.settings.diagnostics {
            if *line_idx == self.current_line {
                res.push((range.clone(), self.settings.colors.diagnostic_color));
            }
        }
        if !self.cache.contains_key(&self.current_line) {
            self.cache.insert(self.current_line, res.clone());
        }
        self.current_line += 1;
        res.into_iter()
    }

    fn current_line(&self) -> usize {
        self.current_line
    }
}
