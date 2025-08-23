use super::Language;

#[derive(Debug, Clone, Copy)]
pub enum SearchText {
    FindPlaceholder,
    FindButton,
    ReplacePlaceholder,
    ReplaceButton,
    ReplaceAllButton,
}

pub fn search_text(key: SearchText, lang: Language) -> &'static str {
    use Language::*;
    match key {
        SearchText::FindPlaceholder => match lang {
            English => "find",
            Russian => "найти",
            _ => "find",
        },
        SearchText::FindButton => match lang {
            English => "Find",
            Russian => "Найти",
            _ => "Find",
        },
        SearchText::ReplacePlaceholder => match lang {
            English => "replace with",
            Russian => "заменить на",
            _ => "replace with",
        },
        SearchText::ReplaceButton => match lang {
            English => "Replace",
            Russian => "Заменить",
            _ => "Replace",
        },
        SearchText::ReplaceAllButton => match lang {
            English => "Replace All",
            Russian => "Заменить все",
            _ => "Replace All",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{search_text, SearchText};
    use crate::app::Language;

    #[test]
    fn search_text_is_translated() {
        assert_eq!(
            search_text(SearchText::FindButton, Language::English),
            "Find"
        );
        assert_eq!(
            search_text(SearchText::FindButton, Language::Russian),
            "Найти"
        );
        assert_eq!(
            search_text(SearchText::ReplaceAllButton, Language::English),
            "Replace All"
        );
        assert_eq!(
            search_text(SearchText::ReplaceAllButton, Language::Russian),
            "Заменить все"
        );
    }
}
