use iced::widget::{column, row, scrollable, text, text_input, MouseArea};
use iced::{theme, Color, Element, Length};
use multicode_core::BlockInfo;

use super::translations::Language;

#[derive(Debug, Clone)]
pub enum PaletteMessage {
    SearchChanged(String),
    StartDrag(usize),
    ToggleFavorite(usize),
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockCategory {
    Arithmetic,
    Conditional,
    Loops,
    Variables,
    Functions,
}

impl BlockCategory {
    pub const ALL: [BlockCategory; 5] = [
        BlockCategory::Arithmetic,
        BlockCategory::Conditional,
        BlockCategory::Loops,
        BlockCategory::Variables,
        BlockCategory::Functions,
    ];

    pub fn title(self, lang: Language) -> &'static str {
        match (self, lang) {
            (BlockCategory::Arithmetic, Language::Russian) => "Арифметика",
            (BlockCategory::Conditional, Language::Russian) => "Условия",
            (BlockCategory::Loops, Language::Russian) => "Циклы",
            (BlockCategory::Variables, Language::Russian) => "Переменные",
            (BlockCategory::Functions, Language::Russian) => "Функции",
            (BlockCategory::Arithmetic, Language::English) => "Arithmetic",
            (BlockCategory::Conditional, Language::English) => "Condition",
            (BlockCategory::Loops, Language::English) => "Loops",
            (BlockCategory::Variables, Language::English) => "Variables",
            (BlockCategory::Functions, Language::English) => "Functions",
        }
    }

    pub fn matches(self, kind: &str) -> bool {
        match (self, kind) {
            (BlockCategory::Arithmetic, "Add")
            | (BlockCategory::Arithmetic, "Subtract")
            | (BlockCategory::Arithmetic, "Multiply")
            | (BlockCategory::Arithmetic, "Divide") => true,
            (BlockCategory::Conditional, "If")
            | (BlockCategory::Conditional, "ElseIf")
            | (BlockCategory::Conditional, "Else") => true,
            (BlockCategory::Loops, "For")
            | (BlockCategory::Loops, "While")
            | (BlockCategory::Loops, "Loop") => true,
            (BlockCategory::Variables, "Set") | (BlockCategory::Variables, "Get") => true,
            (BlockCategory::Functions, "Define")
            | (BlockCategory::Functions, "Call")
            | (BlockCategory::Functions, "Return") => true,
            _ => false,
        }
    }
}

pub struct BlockPalette<'a> {
    blocks: &'a [BlockInfo],
    favorites: &'a [String],
    query: &'a str,
    language: Language,
}

impl<'a> BlockPalette<'a> {
    pub fn new(
        blocks: &'a [BlockInfo],
        favorites: &'a [String],
        query: &'a str,
        language: Language,
    ) -> Self {
        Self {
            blocks,
            favorites,
            query,
            language,
        }
    }

    fn filter_indices(&self) -> Vec<usize> {
        let q = self.query.trim().to_lowercase();
        self.blocks
            .iter()
            .enumerate()
            .filter_map(|(i, b)| {
                if q.is_empty() || matches_block(b, &q) {
                    Some(i)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn view(self) -> Element<'a, PaletteMessage> {
        let search = text_input("search", self.query).on_input(PaletteMessage::SearchChanged);

        let indices = self.filter_indices();
        let mut col = column![];

        if !self.favorites.is_empty() {
            let fav_blocks: Vec<_> = indices
                .iter()
                .copied()
                .filter(|i| self.favorites.contains(&self.blocks[*i].kind))
                .collect();
            if !fav_blocks.is_empty() {
                let title = if self.language == Language::Russian {
                    "Избранное"
                } else {
                    "Favorites"
                };
                col = col.push(text(title));
                for i in fav_blocks {
                    let name = self.blocks[i]
                        .translations
                        .get(self.language.code())
                        .cloned()
                        .unwrap_or_else(|| self.blocks[i].kind.clone());
                    let fav = self.favorites.contains(&self.blocks[i].kind);
                    let star = if fav { "★" } else { "☆" };
                    let star_txt = text(star);
                    let star_txt = if fav {
                        star_txt.style(theme::Text::Color(Color::from_rgb(1.0, 0.8, 0.0)))
                    } else {
                        star_txt
                    };
                    col = col.push(
                        row![
                            MouseArea::new(star_txt).on_press(PaletteMessage::ToggleFavorite(i)),
                            MouseArea::new(text(name)).on_press(PaletteMessage::StartDrag(i)),
                        ]
                        .spacing(5),
                    );
                }
            }
        }

        for cat in BlockCategory::ALL.iter().copied() {
            let cat_blocks: Vec<_> = indices
                .iter()
                .copied()
                .filter(|i| cat.matches(&self.blocks[*i].kind))
                .collect();
            if !cat_blocks.is_empty() {
                col = col.push(text(cat.title(self.language)));
                for i in cat_blocks {
                    let name = self.blocks[i]
                        .translations
                        .get(self.language.code())
                        .cloned()
                        .unwrap_or_else(|| self.blocks[i].kind.clone());
                    let fav = self.favorites.contains(&self.blocks[i].kind);
                    let star = if fav { "★" } else { "☆" };
                    let star_txt = text(star);
                    let star_txt = if fav {
                        star_txt.style(theme::Text::Color(Color::from_rgb(1.0, 0.8, 0.0)))
                    } else {
                        star_txt
                    };
                    col = col.push(
                        row![
                            MouseArea::new(star_txt).on_press(PaletteMessage::ToggleFavorite(i)),
                            MouseArea::new(text(name)).on_press(PaletteMessage::StartDrag(i)),
                        ]
                        .spacing(5),
                    );
                }
            }
        }

        let list = scrollable(col.spacing(5)).height(Length::Fixed(300.0));
        column![search, list].spacing(10).into()
    }
}

fn matches_block(block: &BlockInfo, q: &str) -> bool {
    let en = block
        .translations
        .get("en")
        .map(|s| s.to_lowercase())
        .unwrap_or_default();
    let ru = block
        .translations
        .get("ru")
        .map(|s| s.to_lowercase())
        .unwrap_or_default();
    en.contains(q) || ru.contains(q) || block.kind.to_lowercase().contains(q)
}
