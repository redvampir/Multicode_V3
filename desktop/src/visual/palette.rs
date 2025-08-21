use iced::widget::{column, row, scrollable, text, text_input, MouseArea};
use iced::{theme, Color, Element, Length};
use multicode_core::BlockInfo;

use super::translations::Language;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct PaletteBlock {
    pub info: BlockInfo,
    lower_en: String,
    lower_ru: String,
    lower_kind: String,
}

impl PaletteBlock {
    pub fn new(info: BlockInfo) -> Self {
        let lower_en = info
            .translations
            .get("en")
            .map(|s| s.to_lowercase())
            .unwrap_or_default();
        let lower_ru = info
            .translations
            .get("ru")
            .map(|s| s.to_lowercase())
            .unwrap_or_default();
        let lower_kind = info.kind.to_lowercase();
        Self {
            info,
            lower_en,
            lower_ru,
            lower_kind,
        }
    }
}

#[derive(Debug, Clone)]
pub enum PaletteMessage {
    SearchChanged(String),
    StartDrag(usize),
    ToggleFavorite(usize),
    Close,
}

pub const DEFAULT_CATEGORY: &str = "Other";

fn category_title(cat: &str, lang: Language) -> String {
    match (cat, lang) {
        ("Arithmetic", Language::Russian) => "Арифметика".into(),
        ("Conditional", Language::Russian) | ("Condition", Language::Russian) => "Условия".into(),
        ("Loops", Language::Russian) | ("Loop", Language::Russian) => "Циклы".into(),
        ("Variables", Language::Russian) | ("Variable", Language::Russian) => "Переменные".into(),
        ("Functions", Language::Russian) | ("Function", Language::Russian) => "Функции".into(),
        ("Other", Language::Russian) => "Прочее".into(),
        ("Arithmetic", Language::English) => "Arithmetic".into(),
        ("Conditional", Language::English) | ("Condition", Language::English) => "Condition".into(),
        ("Loops", Language::English) | ("Loop", Language::English) => "Loops".into(),
        ("Variables", Language::English) | ("Variable", Language::English) => "Variables".into(),
        ("Functions", Language::English) | ("Function", Language::English) => "Functions".into(),
        ("Other", Language::English) => "Other".into(),
        _ => cat.to_string(),
    }
}

pub struct BlockPalette<'a> {
    blocks: &'a [PaletteBlock],
    categories: &'a [(String, Vec<usize>)],
    favorites: &'a [String],
    query: &'a str,
    language: Language,
}

impl<'a> BlockPalette<'a> {
    pub fn new(
        blocks: &'a [PaletteBlock],
        categories: &'a [(String, Vec<usize>)],
        favorites: &'a [String],
        query: &'a str,
        language: Language,
    ) -> Self {
        Self {
            blocks,
            categories,
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
        let index_set: HashSet<_> = indices.iter().copied().collect();
        let mut col = column![];

        if !self.favorites.is_empty() {
            let fav_blocks: Vec<_> = indices
                .iter()
                .copied()
                .filter(|i| self.favorites.contains(&self.blocks[*i].info.kind))
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
                        .info
                        .translations
                        .get(self.language.code())
                        .cloned()
                        .unwrap_or_else(|| self.blocks[i].info.kind.clone());
                    let fav = self.favorites.contains(&self.blocks[i].info.kind);
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

        for (cat, cat_indices) in self.categories.iter() {
            let cat_blocks: Vec<_> = cat_indices
                .iter()
                .copied()
                .filter(|i| index_set.contains(i))
                .collect();
            if !cat_blocks.is_empty() {
                col = col.push(text(category_title(cat, self.language)));
                for i in cat_blocks {
                    let name = self.blocks[i]
                        .info
                        .translations
                        .get(self.language.code())
                        .cloned()
                        .unwrap_or_else(|| self.blocks[i].info.kind.clone());
                    let fav = self.favorites.contains(&self.blocks[i].info.kind);
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
        let content = column![search, list].spacing(10);

        MouseArea::new(content)
            .on_press(PaletteMessage::Close)
            .into()
    }
}

fn matches_block(block: &PaletteBlock, q: &str) -> bool {
    block.lower_en.contains(q) || block.lower_ru.contains(q) || block.lower_kind.contains(q)
}
