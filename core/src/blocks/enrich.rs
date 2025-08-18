use std::collections::HashMap;

use crate::{i18n, meta::read_all, parser::Block, BlockInfo};

/// Объединяет исходные `blocks` с метаданными, извлечёнными из `content`.
///
/// Каждый блок получает базовые переводы в зависимости от своего типа и
/// дополняется позиционными и пользовательскими метаданными, если они есть.
pub fn enrich_blocks(blocks: Vec<Block>, content: &str) -> Vec<BlockInfo> {
    let metas = read_all(content);
    let map: HashMap<_, _> = metas.into_iter().map(|m| (m.id.clone(), m)).collect();

    blocks
        .into_iter()
        .map(|b| {
            let label = normalize_kind(&b.kind);
            let mut translations = i18n::lookup(&label).unwrap_or_else(|| {
                let mut m = HashMap::new();
                m.insert("ru".into(), label.clone());
                m.insert("en".into(), label.clone());
                m.insert("es".into(), label.clone());
                m
            });
            if let Some(meta) = map.get(&b.visual_id) {
                translations.extend(meta.translations.clone());
            }
            let pos = map.get(&b.visual_id);
            BlockInfo {
                visual_id: b.visual_id,
                node_id: Some(b.node_id),
                kind: label,
                translations,
                range: (b.range.start, b.range.end),
                anchors: b.anchors.clone(),
                x: pos.map(|m| m.x).unwrap_or(0.0),
                y: pos.map(|m| m.y).unwrap_or(0.0),
                ai: pos.and_then(|m| m.ai.clone()),
                links: pos.map(|m| m.links.clone()).unwrap_or_default(),
            }
        })
        .collect()
}

fn normalize_kind(kind: &str) -> String {
    let lower = kind.to_lowercase();
    if lower == "function/define" {
        "Function/Define".into()
    } else if lower == "function/call" {
        "Function/Call".into()
    } else if lower == "return" {
        "Return".into()
    } else if lower.contains("function") {
        "Function".into()
    } else if lower.contains("if") {
        "Condition".into()
    } else if lower.contains("for") || lower.contains("while") || lower.contains("loop") {
        "Loop".into()
    } else if lower.contains("identifier") || lower.contains("variable") {
        "Variable".into()
    } else if lower.contains("map") {
        "Map".into()
    } else {
        kind.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Block;

    #[test]
    fn enrich_without_metadata() {
        let block = Block {
            visual_id: "1".into(),
            node_id: 1,
            kind: "function".into(),
            range: 0..5,
            anchors: vec![],
        };
        let res = enrich_blocks(vec![block], "");
        assert_eq!(res.len(), 1);
        let b = &res[0];
        assert_eq!(b.x, 0.0);
        assert_eq!(b.y, 0.0);
        assert!(b.translations.get("en").is_some());
    }

    #[test]
    fn enrich_with_metadata() {
        let block = Block {
            visual_id: "42".into(),
            node_id: 1,
            kind: "function".into(),
            range: 0..5,
            anchors: vec![],
        };
        let content = "<!-- @VISUAL_META {\"id\":\"42\",\"x\":1.0,\"y\":2.0,\"translations\":{\"en\":\"Test\"}} -->";
        let res = enrich_blocks(vec![block], content);
        assert_eq!(res.len(), 1);
        let b = &res[0];
        assert_eq!(b.x, 1.0);
        assert_eq!(b.y, 2.0);
        assert_eq!(b.translations.get("en").unwrap(), "Test");
    }
}
