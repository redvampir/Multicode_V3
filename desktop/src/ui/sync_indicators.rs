use crate::sync::SyncEngine;

/// Return a human readable synchronization status.
pub fn status_text(engine: &SyncEngine) -> String {
    let conflicts = engine.last_conflicts();
    if conflicts.is_empty() {
        "Synced".into()
    } else {
        format!("Conflicts: {}", conflicts.len())
    }
}

/// Collect ranges in the source code corresponding to current conflicts.
pub fn conflict_ranges(engine: &SyncEngine) -> Vec<std::ops::Range<usize>> {
    engine
        .last_conflicts()
        .iter()
        .filter_map(|c| engine.range_of(&c.id))
        .collect()
}

/// Lines containing conflicts in the current source code.
pub fn conflict_lines(engine: &SyncEngine) -> Vec<usize> {
    let code = engine.state().code.as_str();
    conflict_ranges(engine)
        .into_iter()
        .map(|range| code[..range.start].lines().count())
        .collect()
}
