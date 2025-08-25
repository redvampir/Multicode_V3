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
use super::conflict_resolver::{ConflictResolver, ResolutionOption};
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
                let parsed = meta::read_all(&code);
                let mut new_code = code.clone();
                let mut metas_map = HashMap::new();
                for mut m in parsed {
                    if let Some(old) = self.state.metas.get(&m.id) {
                        if old.version != m.version {
                            let (option, _) = ConflictResolver::resolve(&m, old);
                            m = match option {
                                ResolutionOption::UseText(t) => t,
                                ResolutionOption::UseVisual(v) => {
                                    new_code = meta::upsert(&new_code, &v);
                                    v
                                }
                                ResolutionOption::Merge(merged) => {
                                    new_code = meta::upsert(&new_code, &merged);
                                    merged
                                }
                            };
                        }
                    }
                    metas_map.insert(m.id.clone(), m);
                }
                self.state.metas = metas_map;
                self.state.code = new_code;
                let metas_vec: Vec<_> = self.state.metas.values().cloned().collect();
                self.state.syntax = self.parser.parse(&self.state.code, &metas_vec);
                Some((self.state.code.clone(), metas_vec))
            }
            SyncMessage::VisualChanged(mut meta) => {
                if meta.version == 0 {
                    meta.version = DEFAULT_VERSION;
                }
                if let Some(text_meta) = self.state.metas.get(&meta.id).cloned() {
                    if text_meta.version != meta.version {
                        let (option, _) = ConflictResolver::resolve(&text_meta, &meta);
                        meta = match option {
                            ResolutionOption::UseText(t) => t,
                            ResolutionOption::UseVisual(v) => v,
                            ResolutionOption::Merge(m) => m,
                        };
                    }
                }
                self.state.code = meta::upsert(&self.state.code, &meta);
                self.state.metas.insert(meta.id.clone(), meta.clone());
                let metas_vec: Vec<_> = self.state.metas.values().cloned().collect();
                self.state.syntax = self.parser.parse(&self.state.code, &metas_vec);
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
}
