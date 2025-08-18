use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Начальное значение версии схемы метаданных.
pub const DEFAULT_VERSION: u32 = 1;

fn default_version() -> u32 {
    DEFAULT_VERSION
}

/// Дополнительные заметки, предоставленные ИИ.
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct AiNote {
    /// Необязательное описание, созданное ИИ.
    pub description: Option<String>,
    /// Необязательные подсказки для пользователя.
    #[serde(default)]
    pub hints: Vec<String>,
}

/// Метаданные, хранящиеся в комментариях `@VISUAL_META`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct VisualMeta {
    /// Версия схемы этих метаданных.
    #[serde(default = "default_version")]
    pub version: u32,
    /// Идентификатор, связывающий эти метаданные с узлами AST.
    pub id: String,
    /// Координата X на холсте.
    pub x: f64,
    /// Координата Y на холсте.
    pub y: f64,
    /// Необязательные теги, связанные с этим блоком.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Необязательные ссылки на другие блоки.
    #[serde(default)]
    pub links: Vec<String>,
    /// Необязательные anchor'ы, связанные с блоком.
    #[serde(default)]
    pub anchors: Vec<String>,
    /// Необязательные команды тестов для запуска этого блока.
    #[serde(default)]
    pub tests: Vec<String>,
    /// Необязательный идентификатор базовой метаданы, которую расширяет запись.
    #[serde(default)]
    pub extends: Option<String>,
    /// Необязательный обратный путь к исходному внешнему файлу.
    #[serde(default)]
    pub origin: Option<String>,
    /// Необязательные переводы меток блоков.
    #[serde(default)]
    pub translations: HashMap<String, String>,
    /// Необязательная заметка, созданная ИИ.
    #[serde(default)]
    pub ai: Option<AiNote>,
    /// Необязательные метаданные, специфичные для плагина.
    #[serde(default)]
    pub extras: Option<Value>,
    /// Метка времени последнего обновления в UTC.
    #[serde(default)]
    pub updated_at: DateTime<Utc>,
}

