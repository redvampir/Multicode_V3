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
//! use desktop::sync::{SyncSettings, SyncEngine, SyncMessage};
//!
//! use multicode_core::parser::Lang;
//!
//! let mut engine = SyncEngine::new(Lang::Rust, SyncSettings::default());
//! let _ = engine.handle(SyncMessage::TextChanged("fn main() {}".into(), Lang::Rust));
//! // далее полученные данные могут быть переданы визуальному редактору
//! ```
//!
//! Дополнительные детали описаны в `docs/sync.md`.

use super::ast_parser::{ASTParser, SyntaxTree};
use super::conflict_resolver::{
    ConflictResolver, ConflictType, ResolutionOption, ResolutionPolicy, SyncConflict,
};
use super::element_mapper::ElementMapper;
use super::settings::SyncSettings;
use super::{
    generate_with_extensions, init_extensions, parse_with_extensions, resolve_with_extensions,
};
use multicode_core::meta::{self, VisualMeta, DEFAULT_VERSION};
use multicode_core::parser::Lang;
use std::collections::{HashMap, HashSet};

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
    /// Добавлена новая связь между блоками.
    ConnectionAdded(String, String),
    /// Сбросить состояние синхронизации.
    ResetSync,
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
    preserve_meta_formatting: bool,
    /// Последние полученные диагностические данные.
    last_diagnostics: SyncDiagnostics,
    /// Последний возвращённый список метаданных.
    last_metas: Vec<VisualMeta>,
    /// Conflicts detected during the last synchronization cycle.
    last_conflicts: Vec<SyncConflict>,
}

impl SyncEngine {
    /// Создаёт новый движок синхронизации.
    pub fn new(lang: Lang, settings: SyncSettings) -> Self {
        init_extensions(None);
        Self {
            state: SyncState::default(),
            parser: ASTParser::new(lang),
            lang,
            last_text_ids: Vec::new(),
            last_visual_ids: Vec::new(),
            mapper: ElementMapper::default(),
            policy: settings.conflict_resolution.into(),
            preserve_meta_formatting: settings.preserve_meta_formatting,
            last_diagnostics: SyncDiagnostics::default(),
            last_metas: Vec::new(),
            last_conflicts: Vec::new(),
        }
    }

    /// Update synchronization behaviour based on new settings.
    pub fn update_settings(&mut self, settings: SyncSettings) {
        self.policy = settings.conflict_resolution.into();
        self.preserve_meta_formatting = settings.preserve_meta_formatting;
    }

    /// Обрабатывает входящее сообщение синхронизации.
    /// Возвращает обновлённый текст, список метаданных и диагностические данные.
    pub fn handle(&mut self, msg: SyncMessage) -> Option<(&str, &[VisualMeta], &SyncDiagnostics)> {
        self.last_conflicts.clear();
        match msg {
            SyncMessage::TextChanged(code, lang) => {
                if self.lang != lang {
                    self.lang = lang;
                    self.parser = ASTParser::new(lang);
                }
                if let Some(ext_metas) = parse_with_extensions(&code, lang) {
                    self.state.metas = ext_metas.into_iter().map(|m| (m.id.clone(), m)).collect();
                    self.state.code = code;
                    self.last_metas = self.state.metas.values().cloned().collect();
                    let metas = std::mem::take(&mut self.last_metas);
                    let diagnostics = self.update_syntax_and_mapper(&metas);
                    self.last_diagnostics = diagnostics;
                    self.last_metas = metas;
                    return Some((&self.state.code, &self.last_metas, &self.last_diagnostics));
                }
                let mut metas = std::mem::take(&mut self.state.metas);
                let resolver = ConflictResolver::default();
                let mut ids = HashSet::new();
                let metas_from_code = match std::panic::catch_unwind(|| meta::read_all(&code)) {
                    Ok(metas) => metas,
                    Err(_) => {
                        self.state.metas.clear();
                        self.state.code = code;
                        self.last_metas.clear();
                        let mut diagnostics = self.update_syntax_and_mapper(&[]);
                        diagnostics.orphaned_blocks.push("malformed meta".into());
                        self.last_diagnostics = diagnostics;
                        return Some((&self.state.code, &self.last_metas, &self.last_diagnostics));
                    }
                };
                for mut m in metas_from_code {
                    if let Some(old) = metas.get(&m.id) {
                        if old.version != m.version {
                            if let Some(resolved) = resolve_with_extensions(old, &m) {
                                m = resolved;
                            } else {
                                let (resolved, conflict) = resolver.resolve(&m, old, self.policy);
                                self.last_conflicts.push(conflict.clone());
                                match conflict.conflict_type {
                                    ConflictType::Structural => tracing::warn!(
                                        id = %conflict.id,
                                        conflict_type = ?conflict.conflict_type,
                                        "Conflict resolved",
                                    ),
                                    _ => tracing::debug!(
                                        id = %conflict.id,
                                        conflict_type = ?conflict.conflict_type,
                                        "Conflict resolved",
                                    ),
                                }
                                m = resolved;
                            }
                        }
                    }
                    ids.insert(m.id.clone());
                    metas.insert(m.id.clone(), m);
                }
                metas.retain(|id, _| ids.contains(id));
                self.state.metas = metas;
                self.state.code = code;
                self.last_metas = self.state.metas.values().cloned().collect();
                let metas = std::mem::take(&mut self.last_metas);
                let diagnostics = self.update_syntax_and_mapper(&metas);
                self.last_diagnostics = diagnostics;
                self.last_metas = metas;
                Some((&self.state.code, &self.last_metas, &self.last_diagnostics))
            }
            SyncMessage::VisualChanged(mut meta) => {
                if let Some(code) = generate_with_extensions(&self.state.code, &meta, self.lang) {
                    self.state.code = code;
                    self.state.metas.insert(meta.id.clone(), meta);
                    self.last_metas = self.state.metas.values().cloned().collect();
                    let metas = std::mem::take(&mut self.last_metas);
                    let diagnostics = self.update_syntax_and_mapper(&metas);
                    self.last_diagnostics = diagnostics;
                    self.last_metas = metas;
                    return Some((&self.state.code, &self.last_metas, &self.last_diagnostics));
                }
                if meta.version == 0 {
                    meta.version = DEFAULT_VERSION;
                }
                if let Some(existing) = self.state.metas.get(&meta.id).cloned() {
                    if existing.version != meta.version {
                        if let Some(resolved) = resolve_with_extensions(&existing, &meta) {
                            meta = resolved;
                        } else {
                            let (resolved, conflict) =
                                ConflictResolver::default().resolve(&existing, &meta, self.policy);
                            self.last_conflicts.push(conflict.clone());
                            match conflict.conflict_type {
                                ConflictType::Structural => tracing::warn!(
                                    id = %conflict.id,
                                    conflict_type = ?conflict.conflict_type,
                                    "Conflict resolved",
                                ),
                                _ => tracing::debug!(
                                    id = %conflict.id,
                                    conflict_type = ?conflict.conflict_type,
                                    "Conflict resolved",
                                ),
                            }
                            meta = resolved;
                        }
                    }
                }
                self.state.code =
                    meta::upsert(&self.state.code, &meta, self.preserve_meta_formatting);
                self.state.metas.insert(meta.id.clone(), meta);
                self.last_metas = self.state.metas.values().cloned().collect();
                let metas = std::mem::take(&mut self.last_metas);
                let diagnostics = self.update_syntax_and_mapper(&metas);
                self.last_diagnostics = diagnostics;
                self.last_metas = metas;
                Some((&self.state.code, &self.last_metas, &self.last_diagnostics))
            }
            SyncMessage::ConnectionAdded(from, to) => {
                if let Some(mut meta) = self.state.metas.get(&from).cloned() {
                    if !meta.links.contains(&to) {
                        meta.links.push(to);
                        return self.handle(SyncMessage::VisualChanged(meta));
                    }
                }
                None
            }
            SyncMessage::ResetSync => {
                self.state = SyncState::default();
                self.last_text_ids.clear();
                self.last_visual_ids.clear();
                self.mapper = ElementMapper::default();
                self.last_diagnostics = SyncDiagnostics::default();
                self.last_metas.clear();
                self.last_conflicts.clear();
                Some((&self.state.code, &self.last_metas, &self.last_diagnostics))
            }
        }
    }

    /// Обновляет синтаксическое дерево, `ElementMapper` и возвращает диагностические данные.
    fn update_syntax_and_mapper(&mut self, metas: &[VisualMeta]) -> SyncDiagnostics {
        self.state.syntax = self.parser.parse(&self.state.code, metas);
        self.mapper = ElementMapper::new(&self.state.code, &self.state.syntax, metas);
        self.log_mapping_issues();
        SyncDiagnostics {
            orphaned_blocks: self.mapper.orphaned_blocks.clone(),
            unmapped_code: self.mapper.unmapped_code.clone(),
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

    /// Возвращает последние диагностические данные.
    pub fn last_diagnostics(&self) -> &SyncDiagnostics {
        &self.last_diagnostics
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

    /// Applies a user-selected resolution strategy to a conflict with the given `id`.
    pub fn apply_resolution(&mut self, id: &str, option: ResolutionOption) {
        let text_meta = match meta::read_all(&self.state.code)
            .into_iter()
            .find(|m| m.id == id)
        {
            Some(m) => m,
            None => {
                self.last_conflicts.retain(|c| c.id != id);
                return;
            }
        };
        let visual_meta = match self.state.metas.get(id).cloned() {
            Some(m) => m,
            None => {
                self.last_conflicts.retain(|c| c.id != id);
                return;
            }
        };

        let policy = match option {
            ResolutionOption::Text => ResolutionPolicy::PreferText,
            ResolutionOption::Visual => ResolutionPolicy::PreferVisual,
            ResolutionOption::Merge => self.policy,
        };

        let resolved = match option {
            ResolutionOption::Text => text_meta.clone(),
            ResolutionOption::Visual => visual_meta.clone(),
            ResolutionOption::Merge => {
                let (resolved, _) =
                    ConflictResolver::default().resolve(&text_meta, &visual_meta, policy);
                resolved
            }
        };

        self.state.metas.insert(id.to_string(), resolved.clone());
        self.state.code = meta::upsert(&self.state.code, &resolved, self.preserve_meta_formatting);

        self.last_metas = self.state.metas.values().cloned().collect();
        let metas = std::mem::take(&mut self.last_metas);
        let diagnostics = self.update_syntax_and_mapper(&metas);
        self.last_diagnostics = diagnostics;
        self.last_metas = metas;

        self.last_conflicts.retain(|c| c.id != id);
    }

    /// Conflicts detected during the last synchronization.
    pub fn last_conflicts(&self) -> &[SyncConflict] {
        &self.last_conflicts
    }
}
