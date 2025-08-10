use serde::{Serialize, Deserialize};

/// Description of a block type provided by a plugin.
///
/// Backend uses this information to tell the frontend which visual
/// components should be loaded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDescriptor {
    /// Identifier of the block kind.
    pub kind: String,
    /// Optional human‑readable label for the block.
    pub label: Option<String>,
}

/// Interface implemented by backend plugins.
///
/// Plugins can extend the system with new block kinds or other
/// functionality.  Each plugin must provide a unique name and may return
/// descriptors of additional blocks that should be available on the
/// frontend.
pub trait Plugin: Send + Sync {
    /// Unique plugin name.
    fn name(&self) -> &'static str;

    /// Return block descriptors contributed by this plugin.
    fn blocks(&self) -> Vec<BlockDescriptor>;
}
