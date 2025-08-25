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
//! use multicode_core::parser::Lang;
//!
//! let mut engine = SyncEngine::new(Lang::Rust);
//! engine.handle(SyncMessage::TextChanged("fn main() {}".into(), Lang::Rust));
//! // далее полученные данные могут быть переданы визуальному редактору
//! ```
//!
//! Дополнительные детали описаны в `docs/sync.md`.

use super::ast_parser::{ASTParser, SyntaxTree};
use super::element_mapper::ElementMapper;
use multicode_core::meta::{self, VisualMeta, DEFAULT_VERSION};
use multicode_core::parser::Lang;
use std::collections::HashMap;

/// Состояние синхронизации между текстовым и визуальным представлениями.
#[derive(Debug, Clone, Default)]
pub struct SyncState {
    /// Текущие метаданные, извлечённые из текста, индексированные по идентификатору.
    pub metas: HashMap<String, VisualMeta>,
    /// Последняя версия текста, известная движку.
    pub code: String,
    /// Последнее разобранное синтаксическое дерево.
    pub syntax: SyntaxTree,
}

/// Сообщения для движка синхронизации.
#[derive(Debug, Clone)]
pub enum SyncMessage {
    /// Текст был изменён, необходимо перечитать метаданные. Принимает язык исходного кода.
    TextChanged(String, Lang),
    /// Визуальные метаданные были изменены, нужно обновить текст.
    VisualChanged(VisualMeta),
}

/// Движок, обрабатывающий [`SyncMessage`] и поддерживающий синхронизацию между
/// текстовым и визуальным редакторами. Потоки данных:
/// - [`SyncMessage::TextChanged`] поступает от текстового редактора и приводит к
///   извлечению метаданных для визуального представления;
/// - [`SyncMessage::VisualChanged`] поступает от визуального редактора и
///   приводит к обновлению исходного текста.
#[derive(Debug)]
pub struct SyncEngine {
    state: SyncState,
    parser: ASTParser,
    lang: Lang,
    /// Последние обработанные идентификаторы метаданных из текстового редактора.
    last_text_ids: Vec<String>,
    /// Последние обработанные идентификаторы метаданных из визуального редактора.
    last_visual_ids: Vec<String>,
    mapper: ElementMapper,
}

impl SyncEngine {
    /// Создаёт новый движок синхронизации.
    pub fn new(lang: Lang) -> Self {
        Self {
            state: SyncState::default(),
            parser: ASTParser::new(lang),
            lang,
            last_text_ids: Vec::new(),
            last_visual_ids: Vec::new(),
            mapper: ElementMapper::default(),
        }
    }

    /// Обрабатывает входящее сообщение синхронизации.
    /// Возвращает обновлённый текст и список метаданных.
    pub fn handle(&mut self, msg: SyncMessage) -> Option<(String, Vec<VisualMeta>)> {
        match msg {
            SyncMessage::TextChanged(code, lang) => {
                if self.lang != lang {
                    self.lang = lang;
                    self.parser = ASTParser::new(lang);
                }
                self.state.metas = meta::read_all(&code)
                    .into_iter()
                    .map(|m| (m.id.clone(), m))
                    .collect();
                self.state.code = code;
                let metas_vec: Vec<_> = self.state.metas.values().cloned().collect();
                self.state.syntax = self.parser.parse(&self.state.code, &metas_vec);
                self.mapper = ElementMapper::new(&self.state.code, &self.state.syntax);
                Some((self.state.code.clone(), metas_vec))
            }
            SyncMessage::VisualChanged(mut meta) => {
                if meta.version == 0 {
                    meta.version = DEFAULT_VERSION;
                }
                self.state.code = meta::upsert(&self.state.code, &meta);
                self.state.metas.insert(meta.id.clone(), meta.clone());
                let metas_vec: Vec<_> = self.state.metas.values().cloned().collect();
                self.state.syntax = self.parser.parse(&self.state.code, &metas_vec);
                self.mapper = ElementMapper::new(&self.state.code, &self.state.syntax);
                Some((self.state.code.clone(), metas_vec))
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

    /// Returns the metadata identifier associated with a byte offset.
    pub fn id_at(&self, offset: usize) -> Option<&str> {
        self.mapper.id_at(offset)
    }

    /// Returns the byte range corresponding to the given metadata identifier.
    pub fn range_of(&self, id: &str) -> Option<std::ops::Range<usize>> {
        self.mapper.range_of(id)
    }

    /// Metadata identifiers that don't map to any code block.
    pub fn orphaned_blocks(&self) -> &[String] {
        &self.mapper.orphaned_blocks
    }

    /// Code ranges that don't have associated metadata.
    pub fn unmapped_code(&self) -> &[std::ops::Range<usize>] {
        &self.mapper.unmapped_code
    }
}
