use super::palette::{PaletteBlock, DEFAULT_CATEGORY};

/// Suggest block indices based on the selected block and categories.
pub fn suggest_blocks(
    blocks: &[PaletteBlock],
    categories: &[(String, Vec<usize>)],
    selected: Option<&str>,
) -> Vec<usize> {
    if let Some(kind) = selected {
        for (_, indices) in categories.iter() {
            if indices.iter().any(|&i| blocks[i].info.kind == kind) {
                return indices
                    .iter()
                    .filter(|&&i| blocks[i].info.kind != kind)
                    .copied()
                    .collect();
            }
        }
    }
    categories
        .iter()
        .find(|(cat, _)| cat == DEFAULT_CATEGORY)
        .or_else(|| categories.first())
        .map(|(_, indices)| indices.clone())
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
        let res = suggest_blocks(&blocks, &categories, Some("Add"));
        assert_eq!(res, vec![1]);
    }
}
