pub mod autocomplete;
pub mod code_editor;
pub mod meta_integration;
pub mod syntax_highlighter;

pub use autocomplete::{suggestions, AutocompleteState, Suggestion};
pub use code_editor::CodeEditor;
pub use syntax_highlighter::THEME_SET;

#[cfg(test)]
mod code_editor_tests;
