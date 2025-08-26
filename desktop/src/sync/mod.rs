//! Модуль синхронизации между текстовым и визуальным редакторами.
//!
//! Все сообщения проходят через [`SyncEngine`], который гарантирует,
//! что оба представления остаются согласованными. Типичный поток данных:
//!
//! - изменения текста -> [`SyncEngine`] -> визуальный редактор;
//! - изменения блок-схем -> [`SyncEngine`] -> текстовый редактор.
//!
//! # Пример
//! ```rust
//! use desktop::sync::{SyncSettings, SyncDiagnostics, SyncEngine, SyncMessage};
//! use multicode_core::parser::Lang;
//!
//! let mut engine = SyncEngine::new(Lang::Rust, SyncSettings::default());
//! // текстовый редактор сообщает об изменении
//! let (_code, _metas, _diag) = engine
//!     .handle(SyncMessage::TextChanged("fn main() {}".into(), Lang::Rust))
//!     .unwrap();
//! ```
//!
//! Более подробное описание потоков данных приведено в `docs/sync.md`.

pub mod ast_parser;
pub mod async_manager;
pub mod change_tracker;
pub mod code_generator;
pub mod conflict_resolver;
pub mod element_mapper;
pub mod engine;
pub mod settings;

pub use ast_parser::{ASTParser, SyntaxNode, SyntaxTree};
pub use async_manager::{AsyncManager, DEFAULT_BATCH_DELAY, DEFAULT_CHANNEL_CAPACITY};
pub use change_tracker::{ChangeTracker, TextDelta, VisualDelta};
pub use code_generator::{format_generated_code, CodeGenerator, FormattingStyle};
pub use conflict_resolver::{
    ConflictResolver, ConflictType, ResolutionOption, ResolutionPolicy, SyncConflict,
};
pub use element_mapper::ElementMapper;
pub use engine::{SyncDiagnostics, SyncEngine, SyncMessage, SyncState};
pub use settings::{ConflictResolutionMode, SyncSettings};

#[cfg(test)]
mod engine_tests;
#[cfg(test)]
mod async_manager_tests;
