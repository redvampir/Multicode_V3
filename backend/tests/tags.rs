use backend::meta::{read_all, upsert, VisualMeta};
use chrono::Utc;
use std::collections::HashMap;

#[test]
fn read_tags_from_comment() {
    let src =
        "// @VISUAL_META {\"id\":\"1\",\"x\":1.0,\"y\":2.0,\"tags\":[\"a\",\"b\"]}\nfn main() {}";
    let metas = read_all(src);
    assert_eq!(metas.len(), 1);
    assert_eq!(metas[0].tags, vec!["a", "b"]);
}

#[test]
fn upsert_preserves_tags() {
    let meta = VisualMeta {
        version: 1,
        id: "1".into(),
        x: 0.0,
        y: 0.0,
        tags: vec!["t".into()],
        links: vec![],
        extends: None,
        origin: None,
        translations: HashMap::new(),
        ai: None,
        extras: None,
        updated_at: Utc::now(),
    };
    let updated = upsert("fn main() {}", &meta);
    assert!(updated.contains("\"tags\":[\"t\"]"));
    let metas = read_all(&updated);
    assert_eq!(metas[0].tags, vec!["t"]);
}
