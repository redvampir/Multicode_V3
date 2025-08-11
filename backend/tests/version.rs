use backend::meta::{read_all, upsert, VisualMeta};
use chrono::Utc;
use std::collections::HashMap;

#[test]
fn defaults_to_version_one() {
    let src = "// @VISUAL_META {\"id\":\"1\",\"x\":0.0,\"y\":0.0}\n";
    let metas = read_all(src);
    assert_eq!(metas.len(), 1);
    assert_eq!(metas[0].version, 1);
}

#[test]
fn reads_explicit_version() {
    let src = "// @VISUAL_META {\"id\":\"1\",\"x\":0.0,\"y\":0.0,\"version\":2}\n";
    let metas = read_all(src);
    assert_eq!(metas.len(), 1);
    assert_eq!(metas[0].version, 2);
}

#[test]
fn upsert_preserves_version() {
    let meta = VisualMeta {
        version: 3,
        id: "1".into(),
        x: 0.0,
        y: 0.0,
        tags: vec![],
        links: vec![],
        extends: None,
        origin: None,
        translations: HashMap::new(),
        ai: None,
        extras: None,
        updated_at: Utc::now(),
    };
    let updated = upsert("fn main() {}", &meta);
    assert!(updated.contains("\"version\":3"));
    let metas = read_all(&updated);
    assert_eq!(metas[0].version, 3);
}
