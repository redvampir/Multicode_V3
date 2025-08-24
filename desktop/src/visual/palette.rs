use iced::widget::{column, row, scrollable, text, text_input, MouseArea};
use iced::{theme, Color, Element, Length};
use multicode_core::BlockInfo;

use super::{
    suggestions::{suggest_blocks, SUGGESTION_LIMIT},
    translations::{block_synonyms, Language},
};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct PaletteBlock {
    pub info: BlockInfo,
    lower_en: String,
    lower_ru: String,
    lower_kind: String,
    tags: HashSet<String>,
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
        let tags = info
            .tags
            .iter()
            .map(|t| t.to_lowercase())
            .collect::<HashSet<_>>();
        Self {
            info,
            lower_en,
            lower_ru,
            lower_kind,
            tags,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
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
    selected: Option<&'a str>,
    language: Language,
}

impl<'a> BlockPalette<'a> {
    pub fn new(
        blocks: &'a [PaletteBlock],
        categories: &'a [(String, Vec<usize>)],
        favorites: &'a [String],
        query: &'a str,
        selected: Option<&'a str>,
        language: Language,
    ) -> Self {
        Self {
            blocks,
            categories,
            favorites,
            query,
            selected,
            language,
        }
    }

    fn filter_indices(&self) -> Vec<usize> {
        let q = self.query.trim().to_lowercase();
        let tokens: Vec<_> = q.split_whitespace().collect();
        self.blocks
            .iter()
            .enumerate()
            .filter_map(|(i, b)| {
                if tokens.is_empty() || matches_block(b, &tokens) {
                    Some(i)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn view(self) -> Element<'a, PaletteMessage> {
        let search = text_input("search", self.query).on_input(PaletteMessage::SearchChanged);
        let q = self.query.trim().to_lowercase();
        let tokens: Vec<_> = q.split_whitespace().collect();

        let mut suggestions = suggest_blocks(
            self.blocks,
            self.categories,
            self.selected,
            SUGGESTION_LIMIT,
        );
        if !tokens.is_empty() {
            suggestions.retain(|&i| matches_block(&self.blocks[i], &tokens));
        }
        let suggestion_set: HashSet<_> = suggestions.iter().copied().collect();

        let indices: Vec<_> = self
            .filter_indices()
            .into_iter()
            .filter(|i| !suggestion_set.contains(i))
            .collect();
        let index_set: HashSet<_> = indices.iter().copied().collect();
        let mut col = column![];

        if !suggestions.is_empty() {
            let title = if self.language == Language::Russian {
                "Подсказки"
            } else {
                "Suggestions"
            };
            col = col.push(text(title));
            for i in suggestions {
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

fn matches_block(block: &PaletteBlock, tokens: &[&str]) -> bool {
    let syns = block_synonyms(&block.info.kind);
    tokens.iter().all(|q| {
        if block.lower_en.contains(q)
            || block.lower_ru.contains(q)
            || block.lower_kind.contains(q)
            || block.tags.contains(*q)
            || block.tags.iter().any(|t| t.contains(q))
        {
            return true;
        }
        if let Some(syns) = syns {
            syns.iter().any(|s| s.contains(q))
        } else {
            false
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::advanced::{
        clipboard,
        layout::{Layout, Node},
        renderer::Null,
        widget::Tree,
        Shell, Widget,
    };
    use iced::event::Event;
    use iced::mouse;
    use iced::widget::Space;
    use iced::{Length, Point, Rectangle, Size, Theme};
    use std::collections::{HashMap, HashSet};

    fn make_block(kind: &str, en: &str, ru: &str) -> PaletteBlock {
        let mut translations = HashMap::new();
        translations.insert("en".to_string(), en.to_string());
        translations.insert("ru".to_string(), ru.to_string());
        PaletteBlock::new(BlockInfo {
            visual_id: String::new(),
            node_id: None,
            kind: kind.to_string(),
            translations,
            range: (0, 0),
            anchors: vec![],
            x: 0.0,
            y: 0.0,
            ports: vec![],
            ai: None,
            tags: vec![],
            links: vec![],
        })
    }

    #[test]
    fn matches_block_case_insensitive() {
        let block = make_block("Loop", "Repeat", "Повторение");
        let q = "RePeAt".to_lowercase();
        assert!(matches_block(&block, &[q.as_str()]));
        let q = "ПОВТОР".to_lowercase();
        assert!(matches_block(&block, &[q.as_str()]));
        let q = "LoOp".to_lowercase();
        assert!(matches_block(&block, &[q.as_str()]));
        let q = "unknown".to_string();
        assert!(!matches_block(&block, &[q.as_str()]));
    }

    #[test]
    fn filter_indices_multiple_tokens() {
        let blocks = vec![
            make_block_with_tags("Loop", "Repeat", "Повторение", vec!["control"]),
            make_block_with_tags("Arithmetic", "Add", "Сложение", vec!["math"]),
        ];
        let categories = vec![];
        let favorites = vec![];
        let palette = BlockPalette::new(
            &blocks,
            &categories,
            &favorites,
            "repeat loop control",
            None,
            Language::English,
        );
        assert_eq!(palette.filter_indices(), vec![0]);

        let palette = BlockPalette::new(
            &blocks,
            &categories,
            &favorites,
            "repeat math",
            None,
            Language::English,
        );
        assert!(palette.filter_indices().is_empty());
    }

    #[test]
    fn filter_indices_favorites_and_categories() {
        let blocks = vec![
            make_block("Arithmetic", "Add", "Сложение"),
            make_block("Loop", "Repeat", "Повторение"),
        ];
        let categories = vec![
            ("Arithmetic".to_string(), vec![0]),
            ("Loops".to_string(), vec![1]),
        ];
        let favorites = vec!["Loop".to_string()];
        let palette = BlockPalette::new(
            &blocks,
            &categories,
            &favorites,
            "",
            None,
            Language::English,
        );
        let indices = palette.filter_indices();
        assert_eq!(indices, vec![0, 1]);

        let index_set: HashSet<_> = indices.iter().copied().collect();
        let fav_blocks: Vec<_> = indices
            .iter()
            .copied()
            .filter(|i| favorites.contains(&blocks[*i].info.kind))
            .collect();
        assert_eq!(fav_blocks, vec![1]);

        let mut cat_map = Vec::new();
        for (cat, cat_indices) in categories.iter() {
            let cat_blocks: Vec<_> = cat_indices
                .iter()
                .copied()
                .filter(|i| index_set.contains(i))
                .collect();
            if !cat_blocks.is_empty() {
                cat_map.push((cat.clone(), cat_blocks));
            }
        }
        assert_eq!(
            cat_map,
            vec![
                ("Arithmetic".to_string(), vec![0]),
                ("Loops".to_string(), vec![1]),
            ]
        );
    }

    fn make_block_with_tags(kind: &str, en: &str, ru: &str, tags: Vec<&str>) -> PaletteBlock {
        let mut translations = HashMap::new();
        translations.insert("en".to_string(), en.to_string());
        translations.insert("ru".to_string(), ru.to_string());
        PaletteBlock::new(BlockInfo {
            visual_id: String::new(),
            node_id: None,
            kind: kind.to_string(),
            translations,
            range: (0, 0),
            anchors: vec![],
            x: 0.0,
            y: 0.0,
            ports: vec![],
            ai: None,
            tags: tags.into_iter().map(|s| s.to_string()).collect(),
            links: vec![],
        })
    }

    #[test]
    fn filter_indices_match_tags() {
        let blocks = vec![
            make_block_with_tags("Add", "Add", "Сложить", vec!["math", "math"]),
            make_block_with_tags("Loop", "Repeat", "Повторение", vec!["control"]),
        ];
        let categories = vec![
            ("Arithmetic".to_string(), vec![0]),
            ("Loops".to_string(), vec![1]),
        ];
        let favorites = vec![];
        let palette = BlockPalette::new(
            &blocks,
            &categories,
            &favorites,
            "math",
            None,
            Language::English,
        );
        let indices = palette.filter_indices();
        assert_eq!(indices, vec![0]);
    }

    #[test]
    fn palette_block_tags_are_unique() {
        let block = make_block_with_tags("Add", "Add", "Сложить", vec!["math", "math", "math"]);
        assert_eq!(block.tags.len(), 1);
    }

    #[test]
    fn matches_block_synonyms_multi_language() {
        let add_block = make_block("Add", "Add", "Сложить");
        assert!(matches_block(&add_block, &["sum"]));
        assert!(matches_block(&add_block, &["прибавить"]));
        assert!(matches_block(&add_block, &["sumar"]));
        assert!(matches_block(&add_block, &["hinzufuegen"]));

        let while_block = make_block("While", "While", "Пока");
        assert!(matches_block(&while_block, &["mientras"]));
        assert!(matches_block(&while_block, &["solange"]));
    }

    #[test]
    fn start_drag_message_on_press() {
        let content = Space::new(Length::Fixed(10.0), Length::Fixed(10.0));
        let mut widget = MouseArea::new(content).on_press(PaletteMessage::StartDrag(0));
        let renderer = Null::new();
        let mut tree = Tree::new(&widget as &dyn Widget<PaletteMessage, Theme, Null>);
        let node = Node::new(Size::new(10.0, 10.0));
        let layout = Layout::new(&node);

        let mut messages = Vec::new();
        let mut shell = Shell::new(&mut messages);
        let mut cb = clipboard::Null;
        let cursor = mouse::Cursor::Available(Point { x: 1.0, y: 1.0 });

        widget.on_event(
            &mut tree,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            layout,
            cursor,
            &renderer,
            &mut cb,
            &mut shell,
            &Rectangle {
                x: 0.0,
                y: 0.0,
                width: 10.0,
                height: 10.0,
            },
        );

        assert_eq!(messages, vec![PaletteMessage::StartDrag(0)]);
    }
}
