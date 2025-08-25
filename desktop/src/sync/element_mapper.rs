use std::collections::{HashMap, HashSet};
use std::ops::Range;

use multicode_core::meta::VisualMeta;

use super::SyntaxTree;

/// Maps code positions to visual metadata identifiers and vice versa.
#[derive(Debug, Default)]
pub struct ElementMapper {
    id_to_range: HashMap<String, Range<usize>>,
    ranges: Vec<(Range<usize>, String)>, // sorted by `Range::start`
    /// Metadata entries that couldn't be matched with any AST block.
    pub orphaned_blocks: Vec<String>,
    /// Code ranges that have no corresponding metadata.
    pub unmapped_code: Vec<Range<usize>>,
}

impl ElementMapper {
    /// Builds mappings between text ranges and metadata identifiers.
    pub fn new(_code: &str, syntax: &SyntaxTree, metas: &[VisualMeta]) -> Self {
        let mut meta_ids: HashSet<String> = metas.iter().map(|m| m.id.clone()).collect();
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

        // Sort and merge unmapped code ranges, combining overlapping or adjacent ones
        unmapped_code.sort_by_key(|r| r.start);
        let mut merged: Vec<Range<usize>> = Vec::new();
        for range in unmapped_code {
            if let Some(last) = merged.last_mut() {
                if range.start <= last.end {
                    if range.end > last.end {
                        last.end = range.end;
                    }
                } else {
                    merged.push(range);
                }
            } else {
                merged.push(range);
            }
        }

        orphaned_blocks.sort();

        Self {
            id_to_range,
            ranges,
            orphaned_blocks,
            unmapped_code: merged,
        }
    }

    /// Finds a metadata identifier for the given byte offset.
    pub fn id_at(&self, offset: usize) -> Option<&str> {
        let idx = match self
            .ranges
            .binary_search_by(|(range, _)| range.start.cmp(&offset))
        {
            Ok(i) => i,
            Err(i) if i > 0 => i - 1,
            Err(_) => return None,
        };
        let (range, id) = &self.ranges[idx];
        if offset < range.end {
            Some(id.as_str())
        } else {
            None
        }
    }

    /// Returns the byte range associated with the given metadata identifier.
    pub fn range_of(&self, id: &str) -> Option<Range<usize>> {
        self.id_to_range.get(id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::SyntaxNode;
    use chrono::Utc;
    use multicode_core::meta::VisualMeta;
    use multicode_core::parser::Block;
    use std::collections::HashMap;

    fn meta(id: &str) -> VisualMeta {
        VisualMeta {
            version: 1,
            id: id.to_string(),
            x: 0.0,
            y: 0.0,
            tags: Vec::new(),
            links: Vec::new(),
            anchors: Vec::new(),
            tests: Vec::new(),
            extends: None,
            origin: None,
            translations: HashMap::new(),
            ai: None,
            extras: None,
            updated_at: Utc::now(),
        }
    }

    fn node(range: Range<usize>, id: &str, node_id: u32) -> SyntaxNode {
        SyntaxNode {
            block: Block {
                visual_id: id.to_string(),
                node_id,
                kind: String::new(),
                range,
                anchors: Vec::new(),
            },
            meta: Some(meta(id)),
        }
    }

    fn unmapped(range: Range<usize>, node_id: u32) -> SyntaxNode {
        SyntaxNode {
            block: Block {
                visual_id: String::new(),
                node_id,
                kind: String::new(),
                range,
                anchors: Vec::new(),
            },
            meta: None,
        }
    }

    #[test]
    fn id_at_binary_searches_ranges() {
        let syntax = SyntaxTree {
            nodes: vec![node(0..5, "a", 0), node(10..20, "b", 1)],
        };
        let metas = vec![meta("a"), meta("b")];
        let mapper = ElementMapper::new("", &syntax, &metas);
        assert_eq!(mapper.id_at(3), Some("a"));
        assert_eq!(mapper.id_at(15), Some("b"));
        assert_eq!(mapper.id_at(5), None);
        assert_eq!(mapper.id_at(25), None);
    }

    #[test]
    fn merges_unmapped_code_ranges() {
        let syntax = SyntaxTree {
            nodes: vec![
                unmapped(15..20, 0),
                unmapped(0..5, 1),
                unmapped(5..10, 2),
                unmapped(18..25, 3),
            ],
        };
        let metas: Vec<VisualMeta> = Vec::new();
        let mapper = ElementMapper::new("", &syntax, &metas);
        assert_eq!(mapper.unmapped_code, vec![0..10, 15..25]);
    }
}
