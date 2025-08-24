use serde::{Deserialize, Serialize};

fn default_cache_size() -> usize {
    128
}

fn default_true() -> bool {
    true
}

/// Settings controlling search indexing and caching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchSettings {
    /// Maximum number of cached search results.
    #[serde(default = "default_cache_size")]
    pub cache_size: usize,
    /// Enable building and using search indexes.
    #[serde(default = "default_true")]
    pub use_index: bool,
}

impl Default for SearchSettings {
    fn default() -> Self {
        Self { cache_size: default_cache_size(), use_index: true }
    }
}
