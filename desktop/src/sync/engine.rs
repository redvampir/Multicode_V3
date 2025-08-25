//! Реализация движка синхронизации редакторов.
//!
//! `SyncEngine` принимает сообщения от текстового и визуального редакторов и
//! возвращает обновлённое состояние. Это позволяет поддерживать оба
//! представления в актуальном виде.
//!
//! # Пример использования
//! ```rust
//! use desktop::sync::{SyncEngine, SyncMessage};
//!
//! let mut engine = SyncEngine::new();
//! engine.handle(SyncMessage::TextChanged("fn main() {}".into()));
//! // далее полученные данные могут быть переданы визуальному редактору
//! ```
//!
//! Дополнительные детали описаны в `docs/sync.md`.

use multicode_core::meta::{self, VisualMeta, DEFAULT_VERSION};

/// Состояние синхронизации между текстовым и визуальным представлениями.
#[derive(Debug, Clone, Default)]
pub struct SyncState {
    /// Текущие метаданные, извлечённые из текста.
    pub metas: Vec<VisualMeta>,
    /// Последняя версия текста, известная движку.
    pub code: String,
}

/// Сообщения для движка синхронизации.
#[derive(Debug, Clone)]
pub enum SyncMessage {
    /// Текст был изменён, необходимо перечитать метаданные.
    TextChanged(String),
    /// Визуальные метаданные были изменены, нужно обновить текст.
    VisualChanged(VisualMeta),
}

/// Движок, обрабатывающий [`SyncMessage`] и поддерживающий синхронизацию между
/// текстовым и визуальным редакторами. Потоки данных:
/// - [`SyncMessage::TextChanged`] поступает от текстового редактора и приводит к
///   извлечению метаданных для визуального представления;
/// - [`SyncMessage::VisualChanged`] поступает от визуального редактора и
///   приводит к обновлению исходного текста.
#[derive(Debug, Default)]
pub struct SyncEngine {
    state: SyncState,
    /// Последние обработанные идентификаторы метаданных из текстового редактора.
    last_text_ids: Vec<String>,
    /// Последние обработанные идентификаторы метаданных из визуального редактора.
    last_visual_ids: Vec<String>,
}

impl SyncEngine {
    /// Создаёт новый движок синхронизации.
    pub fn new() -> Self {
        Self::default()
    }

    /// Обрабатывает входящее сообщение синхронизации.
    /// Возвращает обновлённый текст и список метаданных.
    pub fn handle(&mut self, msg: SyncMessage) -> Option<(String, Vec<VisualMeta>)> {
        match msg {
            SyncMessage::TextChanged(code) => {
                self.state.metas = meta::read_all(&code);
                self.state.code = code;
                Some((self.state.code.clone(), self.state.metas.clone()))
            }
            SyncMessage::VisualChanged(mut meta) => {
                if meta.version == 0 {
                    meta.version = DEFAULT_VERSION;
                }
                self.state.code = meta::upsert(&self.state.code, &meta);
                if let Some(existing) = self.state.metas.iter_mut().find(|m| m.id == meta.id) {
                    *existing = meta;
                } else {
                    self.state.metas.push(meta);
                }
                Some((self.state.code.clone(), self.state.metas.clone()))
            }
        }
    }

    /// Возвращает текущее состояние синхронизации.
    pub fn state(&self) -> &SyncState {
        &self.state
    }

    /// Принимает идентификаторы метаданных, изменения которых необходимо
    /// синхронизировать. В текущей реализации идентификаторы лишь сохраняются
    /// во внутреннем состоянии, что позволяет тестам проверять факт передачи
    /// данных.
    pub fn process_changes(&mut self, text_ids: Vec<String>, visual_ids: Vec<String>) {
        self.last_text_ids = text_ids;
        self.last_visual_ids = visual_ids;
    }

    /// Возвращает последние обработанные идентификаторы из текстового редактора.
    pub fn last_text_changes(&self) -> &[String] {
        &self.last_text_ids
    }

    /// Возвращает последние обработанные идентификаторы из визуального редактора.
    pub fn last_visual_changes(&self) -> &[String] {
        &self.last_visual_ids
    }
}
