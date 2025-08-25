pub mod autocomplete;
pub mod code_editor;
pub mod meta_integration;
pub mod syntax_highlighter;
pub mod settings;

pub use autocomplete::{suggestions, AutocompleteState, Suggestion};
pub use code_editor::CodeEditor;
pub use syntax_highlighter::THEME_SET;
pub use settings::{EditorSettings, EditorTheme, CustomTheme};

#[cfg(test)]
mod code_editor_tests;
#[cfg(test)]
mod meta_integration_tests;
