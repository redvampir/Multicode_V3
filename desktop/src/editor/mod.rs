pub mod code_editor;
pub mod autocomplete;
pub mod syntax_highlighter;

pub use code_editor::CodeEditor;
pub use syntax_highlighter::THEME_SET;
pub use autocomplete::{AutocompleteState, Suggestion, suggestions};

#[cfg(test)]
mod code_editor_tests;
