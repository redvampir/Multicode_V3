use backend::meta::{read_all, upsert, VisualMeta};
use backend::search::search_links;
use chrono::Utc;
use std::collections::HashMap;
use std::fs;
use tempfile::tempdir;

#[test]
fn read_links_from_comment() {
    let src =
        "// @VISUAL_META {\"id\":\"1\",\"x\":0.0,\"y\":0.0,\"links\":[\"a\",\"b\"]}\nfn main() {}";
    let metas = read_all(src);
    assert_eq!(metas.len(), 1);
    assert_eq!(metas[0].links, vec!["a", "b"]);
}

#[test]
fn upsert_preserves_links() {
    let meta = VisualMeta {
        version: 1,
        id: "1".into(),
        x: 0.0,
        y: 0.0,
        tags: vec![],
        links: vec!["l".into()],
        extends: None,
        origin: None,
        translations: HashMap::new(),
        ai: None,
        extras: None,
        updated_at: Utc::now(),
    };
    let updated = upsert("fn main() {}", &meta);
    assert!(updated.contains("\"links\":[\"l\"]"));
    let metas = read_all(&updated);
    assert_eq!(metas[0].links, vec!["l"]);
}

#[test]
fn search_finds_by_link() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("a.rs");
    fs::write(
        &file,
        "// @VISUAL_META {\"id\":\"1\",\"x\":0.0,\"y\":0.0,\"links\":[\"target\"]}\n",
    )
    .unwrap();
    let res = search_links(dir.path(), "target");
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].file, file);
    assert_eq!(res[0].meta.links, vec!["target"]);
}
