use backend::meta::{self, id_registry};

#[test]
fn detects_duplicate_ids() {
    id_registry::clear();
    let content = "# @VISUAL_META {\"id\":\"dup\",\"x\":0.0,\"y\":0.0,\"updated_at\":\"2024-01-01T00:00:00Z\"}\n# @VISUAL_META {\"id\":\"dup\",\"x\":1.0,\"y\":1.0,\"updated_at\":\"2024-01-01T00:00:00Z\"}"; 
    // reading registers IDs
    meta::read_all(content);
    let dups = id_registry::duplicates();
    assert_eq!(dups, vec!["dup".to_string()]);
}

#[test]
fn finds_registered_meta() {
    id_registry::clear();
    let content = "# @VISUAL_META {\"id\":\"main\",\"x\":1.0,\"y\":2.0,\"updated_at\":\"2024-01-01T00:00:00Z\"}"; 
    meta::read_all(content);
    let found = id_registry::get("main").expect("meta not found");
    assert_eq!(found.x, 1.0);
    assert_eq!(found.y, 2.0);
}
