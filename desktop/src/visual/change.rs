use crate::sync::VisualDelta;
use multicode_core::meta::VisualMeta;

/// Build a [`VisualDelta`] for a changed visual block.
///
/// The delta references the block by its `meta.id` so the synchronisation
/// layer knows which entry was updated.
pub fn delta_from_meta(meta: &VisualMeta) -> VisualDelta {
    VisualDelta {
        meta_ids: vec![meta.id.clone()],
    }
}
