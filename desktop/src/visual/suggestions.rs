use super::palette::{PaletteBlock, DEFAULT_CATEGORY};
use std::collections::HashSet;

/// Maximum number of suggestions to return.
pub const SUGGESTION_LIMIT: usize = 5;

/// Suggest block indices based on the selected block and categories.
pub fn suggest_blocks(
    blocks: &[PaletteBlock],
    categories: &[(String, Vec<usize>)],
    selected: Option<&str>,
    limit: usize,
) -> Vec<usize> {
    if let Some(kind) = selected {
        let in_categories = categories
            .iter()
            .any(|(_, indices)| indices.iter().any(|&i| blocks[i].info.kind == kind));

        if !in_categories {
            return categories
                .iter()
                .find(|(cat, _)| cat == DEFAULT_CATEGORY)
                .map(|(_, indices)| indices.iter().copied().take(limit).collect())
                .unwrap_or_default();
        }

        if let Some(selected_block) = blocks.iter().find(|b| b.info.kind == kind) {
            let tag_set: HashSet<String> = selected_block
                .info
                .tags
                .iter()
                .map(|t| t.to_lowercase())
                .collect();
            if !tag_set.is_empty() {
                let suggestions: Vec<usize> = blocks
                    .iter()
                    .enumerate()
                    .filter(|(_, b)| {
                        b.info.kind != kind
                            && b.info
                                .tags
                                .iter()
                                .any(|t| tag_set.contains(&t.to_lowercase()))
                    })
                    .map(|(i, _)| i)
                    .take(limit)
                    .collect();
                if !suggestions.is_empty() {
                    return suggestions;
                }
            }
        }
        for (_, indices) in categories.iter() {
            if indices.iter().any(|&i| blocks[i].info.kind == kind) {
                return indices
                    .iter()
                    .filter(|&&i| blocks[i].info.kind != kind)
                    .copied()
                    .take(limit)
                    .collect();
            }
        }
    }
    categories
        .iter()
        .find(|(cat, _)| cat == DEFAULT_CATEGORY)
        .or_else(|| categories.first())
        .map(|(_, indices)| indices.iter().copied().take(limit).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use multicode_core::BlockInfo;
    use std::collections::HashMap;

    fn make_block(kind: &str) -> PaletteBlock {
        let mut translations = HashMap::new();
        translations.insert("en".to_string(), kind.to_string());
        PaletteBlock::new(BlockInfo {
            visual_id: String::new(),
            node_id: None,
            kind: kind.to_string(),
            translations,
            range: (0, 0),
            anchors: vec![],
            x: 0.0,
            y: 0.0,
            ports: vec![],
            ai: None,
            tags: vec![],
            links: vec![],
        })
    }

    fn make_block_with_tags(kind: &str, tags: &[&str]) -> PaletteBlock {
        let mut translations = HashMap::new();
        translations.insert("en".to_string(), kind.to_string());
        PaletteBlock::new(BlockInfo {
            visual_id: String::new(),
            node_id: None,
            kind: kind.to_string(),
            translations,
            range: (0, 0),
            anchors: vec![],
            x: 0.0,
            y: 0.0,
            ports: vec![],
            ai: None,
            tags: tags.iter().map(|t| t.to_string()).collect(),
            links: vec![],
        })
    }

    #[test]
    fn suggests_from_same_category_excluding_selected() {
        let blocks = vec![
            make_block("Add"),
            make_block("Subtract"),
            make_block("Loop"),
        ];
        let categories = vec![
            ("Arithmetic".to_string(), vec![0, 1]),
            ("Loops".to_string(), vec![2]),
        ];
        let res = suggest_blocks(&blocks, &categories, Some("Add"), SUGGESTION_LIMIT);
        assert_eq!(res, vec![1]);
    }

    #[test]
    fn suggests_blocks_with_common_tags() {
        let blocks = vec![
            make_block_with_tags("Add", &["math"]),
            make_block_with_tags("Multiply", &["math"]),
            make_block_with_tags("Loop", &["control"]),
        ];
        let categories = vec![
            ("Arithmetic".to_string(), vec![0]),
            ("Other".to_string(), vec![1]),
            ("Loops".to_string(), vec![2]),
        ];
        let res = suggest_blocks(&blocks, &categories, Some("Add"), SUGGESTION_LIMIT);
        assert_eq!(res, vec![1]);
    }

    #[test]
    fn falls_back_to_category_when_no_common_tags() {
        let blocks = vec![
            make_block_with_tags("Add", &["math1"]),
            make_block_with_tags("Subtract", &["math2"]),
            make_block_with_tags("Loop", &["control"]),
        ];
        let categories = vec![
            ("Arithmetic".to_string(), vec![0, 1]),
            ("Loops".to_string(), vec![2]),
        ];
        let res = suggest_blocks(&blocks, &categories, Some("Add"), SUGGESTION_LIMIT);
        assert_eq!(res, vec![1]);
    }

    #[test]
    fn empty_when_selected_not_in_categories() {
        let blocks = vec![make_block("Add"), make_block("Subtract")];
        let categories = vec![("Arithmetic".to_string(), vec![1])];
        let res = suggest_blocks(&blocks, &categories, Some("Add"), SUGGESTION_LIMIT);
        assert!(res.is_empty());
    }

    #[test]
    fn respects_limit() {
        // Create more suggestions than the limit and ensure the result is truncated.
        let blocks: Vec<_> = (0..10)
            .map(|i| make_block_with_tags(&format!("B{}", i), &["tag"]))
            .collect();
        let categories = vec![("Other".to_string(), (0..10).collect())];
        let res = suggest_blocks(&blocks, &categories, Some("B0"), 3);
        assert!(res.len() <= 3);
    }
}
