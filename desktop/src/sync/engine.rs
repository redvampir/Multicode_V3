//! Реализация движка синхронизации редакторов.
//!
//! `SyncEngine` принимает сообщения от текстового и визуального редакторов и
//! возвращает обновлённое состояние. Это позволяет поддерживать оба
//! представления в актуальном виде.
//!
//! Помимо обмена данными, движок предоставляет API для сопоставления позиций
//! в исходном тексте с идентификаторами визуальных блоков. Методы
//! [`id_at`], [`id_at_position`] и [`range_of`] позволяют находить метаданные по
//! смещению или координатам и наоборот. Также доступны методы
//! [`orphaned_blocks`] и [`unmapped_code`], помогающие выявлять расхождения между
//! кодом и метаданными.
//!
//! # Пример использования
//! ```rust
//! use desktop::sync::{ResolutionPolicy, SyncEngine, SyncMessage};
//!
//! use multicode_core::parser::Lang;
//!
//! let mut engine = SyncEngine::new(Lang::Rust, ResolutionPolicy::PreferText);
//! let _ = engine.handle(SyncMessage::TextChanged("fn main() {}".into(), Lang::Rust));
//! // далее полученные данные могут быть переданы визуальному редактору
//! ```
//!
//! Дополнительные детали описаны в `docs/sync.md`.

use super::ast_parser::{ASTParser, SyntaxTree};
use super::conflict_resolver::{ConflictResolver, ConflictType, ResolutionPolicy};
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

/// Диагностическая информация о сопоставлении кода и метаданных.
#[derive(Debug, Clone, Default)]
pub struct SyncDiagnostics {
    /// Метаданные, для которых не найден соответствующий фрагмент кода.
    pub orphaned_blocks: Vec<String>,
    /// Участки кода, не связанные ни с одним блоком метаданных.
    pub unmapped_code: Vec<std::ops::Range<usize>>,
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
    policy: ResolutionPolicy,
}

impl SyncEngine {
    /// Создаёт новый движок синхронизации.
    pub fn new(lang: Lang, policy: ResolutionPolicy) -> Self {
        Self {
            state: SyncState::default(),
            parser: ASTParser::new(lang),
            lang,
            last_text_ids: Vec::new(),
            last_visual_ids: Vec::new(),
            mapper: ElementMapper::default(),
            policy,
        }
    }

    /// Обрабатывает входящее сообщение синхронизации.
    /// Возвращает обновлённый текст, список метаданных и диагностические данные.
    pub fn handle(
        &mut self,
        msg: SyncMessage,
    ) -> Option<(String, Vec<VisualMeta>, SyncDiagnostics)> {
        match msg {
            SyncMessage::TextChanged(code, lang) => {
                if self.lang != lang {
                    self.lang = lang;
                    self.parser = ASTParser::new(lang);
                }
                let previous = self.state.metas.clone();
                let resolver = ConflictResolver::default();
                let mut map = HashMap::new();
                for mut m in meta::read_all(&code) {
                    if let Some(old) = previous.get(&m.id) {
                        if old.version != m.version {
                            let (resolved, conflict) = resolver.resolve(&m, old, self.policy);
                            match conflict.conflict_type {
                                ConflictType::Structural => tracing::warn!(
                                    id = %conflict.id,
                                    conflict_type = ?conflict.conflict_type,
                                    "Conflict resolved"
                                ),
                                _ => tracing::debug!(
                                    id = %conflict.id,
                                    conflict_type = ?conflict.conflict_type,
                                    "Conflict resolved"
                                ),
                            }
                            m = resolved;
                        }
                    }
                    map.insert(m.id.clone(), m);
                }
                self.state.metas = map;
                self.state.code = code;
                let metas_vec: Vec<_> = self.state.metas.values().cloned().collect();
                self.state.syntax = self.parser.parse(&self.state.code, &metas_vec);
                self.mapper = ElementMapper::new(&self.state.code, &self.state.syntax, &metas_vec);
                self.log_mapping_issues();
                let diagnostics = SyncDiagnostics {
                    orphaned_blocks: self.mapper.orphaned_blocks.clone(),
                    unmapped_code: self.mapper.unmapped_code.clone(),
                };
                Some((self.state.code.clone(), metas_vec, diagnostics))
            }
            SyncMessage::VisualChanged(mut meta) => {
                if meta.version == 0 {
                    meta.version = DEFAULT_VERSION;
                }
                if let Some(existing) = self.state.metas.get(&meta.id).cloned() {
                    if existing.version != meta.version {
                        let (resolved, conflict) =
                            ConflictResolver::default().resolve(&existing, &meta, self.policy);
                        match conflict.conflict_type {
                            ConflictType::Structural => tracing::warn!(
                                id = %conflict.id,
                                conflict_type = ?conflict.conflict_type,
                                "Conflict resolved"
                            ),
                            _ => tracing::debug!(
                                id = %conflict.id,
                                conflict_type = ?conflict.conflict_type,
                                "Conflict resolved"
                            ),
                        }
                        meta = resolved;
                    }
                }
                self.state.code = meta::upsert(&self.state.code, &meta);
                self.state.metas.insert(meta.id.clone(), meta.clone());
                let metas_vec: Vec<_> = self.state.metas.values().cloned().collect();
                self.state.syntax = self.parser.parse(&self.state.code, &metas_vec);
                self.mapper = ElementMapper::new(&self.state.code, &self.state.syntax, &metas_vec);
                self.log_mapping_issues();
                let diagnostics = SyncDiagnostics {
                    orphaned_blocks: self.mapper.orphaned_blocks.clone(),
                    unmapped_code: self.mapper.unmapped_code.clone(),
                };
                Some((self.state.code.clone(), metas_vec, diagnostics))
            }
        }
    }

    /// Logs warnings for orphaned metadata blocks or unmapped code ranges.
    fn log_mapping_issues(&self) {
        if !self.mapper.orphaned_blocks.is_empty() {
            tracing::warn!(
                orphaned = ?self.mapper.orphaned_blocks,
                "Orphaned metadata blocks"
            );
        }
        if !self.mapper.unmapped_code.is_empty() {
            tracing::warn!(
                unmapped = ?self.mapper.unmapped_code,
                "Unmapped code ranges"
            );
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

    /// Returns the metadata identifier associated with a byte offset in the
    /// current source code. This is useful for mapping a cursor position from
    /// the text editor to a visual block.
    pub fn id_at(&self, offset: usize) -> Option<&str> {
        self.mapper.id_at(offset)
    }

    /// Returns the metadata identifier at the given `(line, column)` position
    /// in the source code.
    pub fn id_at_position(&self, line: usize, column: usize) -> Option<&str> {
        self.mapper.id_at_position(&self.state.code, line, column)
    }

    /// Returns the byte range corresponding to the given metadata identifier,
    /// allowing the UI to highlight the code associated with a visual block.
    pub fn range_of(&self, id: &str) -> Option<std::ops::Range<usize>> {
        self.mapper.range_of(id)
    }

    /// Metadata identifiers that don't map to any code block. These indicate
    /// metadata present in the file but missing in the parsed syntax tree.
    pub fn orphaned_blocks(&self) -> &[String] {
        &self.mapper.orphaned_blocks
    }

    /// Code ranges that don't have associated metadata and therefore lack a
    /// corresponding visual block.
    pub fn unmapped_code(&self) -> &[std::ops::Range<usize>] {
        &self.mapper.unmapped_code
    }
}
