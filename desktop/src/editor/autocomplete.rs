use std::collections::{HashMap, HashSet};

use once_cell::sync::Lazy;

pub static KEYWORDS: &[&str] = &[
    "fn", "let", "if", "else", "for", "while", "loop", "match", "struct",
    "enum", "impl", "trait", "return", "break", "continue", "pub", "use",
    "mod", "const", "static",
];

pub static SNIPPETS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    HashMap::from([
        ("fn", "fn name() {\n    \n}"),
        ("if", "if condition {\n    \n}"),
        ("for", "for item in iterator {\n    \n}"),
    ])
});

#[derive(Debug, Clone)]
pub struct Suggestion {
    pub label: String,
    pub insert: String,
}

#[derive(Debug, Clone)]
pub struct AutocompleteState {
    pub suggestions: Vec<Suggestion>,
    pub selected: usize,
}

impl AutocompleteState {
    pub fn new(suggestions: Vec<Suggestion>) -> Self {
        Self { suggestions, selected: 0 }
    }

    pub fn current(&self) -> Option<&Suggestion> {
        self.suggestions.get(self.selected)
    }

    pub fn next(&mut self) {
        if !self.suggestions.is_empty() {
            self.selected = (self.selected + 1) % self.suggestions.len();
        }
    }

    pub fn prev(&mut self) {
        if !self.suggestions.is_empty() {
            if self.selected == 0 {
                self.selected = self.suggestions.len() - 1;
            } else {
                self.selected -= 1;
            }
        }
    }
}

pub fn suggestions(content: &str, prefix: &str) -> Vec<Suggestion> {
    if prefix.is_empty() {
        return Vec::new();
    }
    let mut uniq = HashSet::new();
    let mut items = Vec::new();

    for &kw in KEYWORDS {
        if kw.starts_with(prefix) && uniq.insert(kw) {
            let insert = SNIPPETS.get(kw).unwrap_or(&kw);
            items.push(Suggestion {
                label: kw.to_string(),
                insert: (*insert).to_string(),
            });
        }
    }

    for token in content
        .split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .filter(|t| !t.is_empty())
    {
        if token.starts_with(prefix) && uniq.insert(token) {
            let insert = SNIPPETS.get(token).unwrap_or(&token);
            items.push(Suggestion {
                label: token.to_string(),
                insert: (*insert).to_string(),
            });
        }
    }

    items.sort_by(|a, b| a.label.cmp(&b.label));
    items
}

