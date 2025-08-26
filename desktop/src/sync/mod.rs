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

use once_cell::sync::Lazy;
use std::path::Path;
use std::sync::{Mutex, Once};

use libloading::Library;

use multicode_core::meta::VisualMeta;
use multicode_core::parser::Lang;

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

/// Расширение механизма синхронизации.
///
/// Предоставляет возможность добавлять пользовательскую логику разбора
/// [`VisualMeta`], генерации кода и разрешения конфликтов.
pub trait SyncExtension: Send + Sync {
    /// Попытаться разобрать список [`VisualMeta`] из исходного кода.
    fn parse(&self, _code: &str, _lang: Lang) -> Option<Vec<VisualMeta>> {
        None
    }

    /// Попытаться сгенерировать новый код для заданных метаданных.
    fn generate(&self, _code: &str, _meta: &VisualMeta, _lang: Lang) -> Option<String> {
        None
    }

    /// Разрешить конфликт между текстовой и визуальной версиями метаданных.
    fn resolve(&self, _text: &VisualMeta, _visual: &VisualMeta) -> Option<VisualMeta> {
        None
    }
}

static SYNC_EXTENSIONS: Lazy<Mutex<Vec<Box<dyn SyncExtension>>>> =
    Lazy::new(|| Mutex::new(Vec::new()));
static LOADED_LIBS: Lazy<Mutex<Vec<Library>>> = Lazy::new(|| Mutex::new(Vec::new()));
static INIT: Once = Once::new();

/// Зарегистрировать расширение синхронизации.
pub fn register_extension<E: SyncExtension + 'static>(ext: E) {
    if let Ok(mut exts) = SYNC_EXTENSIONS.lock() {
        exts.push(Box::new(ext));
    }
}

fn register_boxed_extension(ext: Box<dyn SyncExtension>) {
    if let Ok(mut exts) = SYNC_EXTENSIONS.lock() {
        exts.push(ext);
    }
}

/// Инициализировать расширения, загружая их из каталога `plugins/`.
pub fn init_extensions() {
    INIT.call_once(load_extensions);
}

fn load_extensions() {
    if let Ok(entries) = std::fs::read_dir("plugins") {
        for entry in entries.flatten() {
            let path = entry.path();
            match path.extension().and_then(|e| e.to_str()) {
                Some("dll") | Some("so") | Some("dylib") => unsafe {
                    if let Some((ext, lib)) = load_dll(&path) {
                        register_boxed_extension(ext);
                        if let Ok(mut libs) = LOADED_LIBS.lock() {
                            libs.push(lib);
                        }
                    }
                },
                _ => {}
            }
        }
    }
}

unsafe fn load_dll(path: &Path) -> Option<(Box<dyn SyncExtension>, Library)> {
    use libloading::Symbol;
    type Constructor = unsafe fn() -> Box<dyn SyncExtension>;

    let lib = Library::new(path).ok()?;
    let constructor: Symbol<Constructor> = lib.get(b"create_extension").ok()?;
    let ext = constructor();
    Some((ext, lib))
}

/// Использовать зарегистрированные расширения для парсинга кода.
pub(crate) fn parse_with_extensions(code: &str, lang: Lang) -> Option<Vec<VisualMeta>> {
    if let Ok(exts) = SYNC_EXTENSIONS.lock() {
        for ext in exts.iter() {
            if let Some(m) = ext.parse(code, lang) {
                return Some(m);
            }
        }
    }
    None
}

/// Использовать зарегистрированные расширения для генерации кода.
pub(crate) fn generate_with_extensions(
    code: &str,
    meta: &VisualMeta,
    lang: Lang,
) -> Option<String> {
    if let Ok(exts) = SYNC_EXTENSIONS.lock() {
        for ext in exts.iter() {
            if let Some(c) = ext.generate(code, meta, lang) {
                return Some(c);
            }
        }
    }
    None
}

/// Использовать зарегистрированные расширения для разрешения конфликтов.
pub(crate) fn resolve_with_extensions(
    text: &VisualMeta,
    visual: &VisualMeta,
) -> Option<VisualMeta> {
    if let Ok(exts) = SYNC_EXTENSIONS.lock() {
        for ext in exts.iter() {
            if let Some(resolved) = ext.resolve(text, visual) {
                return Some(resolved);
            }
        }
    }
    None
}

#[cfg(test)]
mod async_manager_tests;
#[cfg(test)]
mod engine_tests;
