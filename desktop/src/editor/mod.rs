pub mod code_editor;
pub mod autocomplete;

pub use code_editor::CodeEditor;
pub use code_editor::THEME_SET;
pub use autocomplete::{AutocompleteState, Suggestion, suggestions};

#[cfg(test)]
mod code_editor_tests;
