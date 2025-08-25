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
//! use desktop::sync::{SyncEngine, SyncMessage};
//!
//! let mut engine = SyncEngine::new();
//! // текстовый редактор сообщает об изменении
//! let (_code, _metas) = engine
//!     .handle(SyncMessage::TextChanged("fn main() {}".into()))
//!     .unwrap();
//! ```
//!
//! Более подробное описание потоков данных приведено в `docs/sync.md`.

pub mod engine;

pub use engine::{SyncEngine, SyncMessage, SyncState};

#[cfg(test)]
mod engine_tests;
