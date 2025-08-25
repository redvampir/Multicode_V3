use std::collections::{HashMap, HashSet};
use std::ops::Range;

use multicode_core::meta;

use super::SyntaxTree;

/// Maps code positions to visual metadata identifiers and vice versa.
#[derive(Debug, Default)]
pub struct ElementMapper {
    id_to_range: HashMap<String, Range<usize>>,
    ranges: Vec<(Range<usize>, String)>,
    /// Metadata entries that couldn't be matched with any AST block.
    pub orphaned_blocks: Vec<String>,
    /// Code ranges that have no corresponding metadata.
    pub unmapped_code: Vec<Range<usize>>,
}

impl ElementMapper {
    /// Builds mappings between text ranges and metadata identifiers.
    pub fn new(code: &str, syntax: &SyntaxTree) -> Self {
        let metas = meta::read_all(code);
        let mut meta_ids: HashSet<String> = metas.into_iter().map(|m| m.id).collect();
        let mut id_to_range = HashMap::new();
        let mut ranges = Vec::new();
        let mut unmapped_code = Vec::new();

        for node in &syntax.nodes {
            if let Some(meta) = &node.meta {
                let id = meta.id.clone();
                id_to_range.insert(id.clone(), node.block.range.clone());
                ranges.push((node.block.range.clone(), id.clone()));
                meta_ids.remove(&id);
            } else {
                unmapped_code.push(node.block.range.clone());
            }
        }

        let mut orphaned_blocks: Vec<String> = meta_ids.into_iter().collect();
        ranges.sort_by_key(|(r, _)| r.start);
        orphaned_blocks.sort();

        Self {
            id_to_range,
            ranges,
            orphaned_blocks,
            unmapped_code,
        }
    }

    /// Finds a metadata identifier for the given byte offset.
    pub fn id_at(&self, offset: usize) -> Option<&str> {
        for (range, id) in &self.ranges {
            if range.start <= offset && offset < range.end {
                return Some(id.as_str());
            }
        }
        None
    }

    /// Returns the byte range associated with the given metadata identifier.
    pub fn range_of(&self, id: &str) -> Option<Range<usize>> {
        self.id_to_range.get(id).cloned()
    }
}
