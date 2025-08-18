use core::meta::{self, id_registry};

#[test]
fn detects_duplicate_ids() {
    let content = "# @VISUAL_META {\"id\":\"dup\",\"x\":0.0,\"y\":0.0}\n# @VISUAL_META {\"id\":\"dup\",\"x\":1.0,\"y\":1.0}";
    // чтение регистрирует ID и фиксирует дубликаты
    let (_metas, dups) = meta::read_all_with_dups(content);
    assert_eq!(dups, vec!["dup".to_string()]);
}

#[test]
fn finds_registered_meta() {
    id_registry::clear();
    let content = "# @VISUAL_META {\"id\":\"main\",\"x\":1.0,\"y\":2.0}";
    meta::read_all(content);
    let found = id_registry::get("main").expect("метаданные не найдены");
    assert_eq!(found.x, 1.0);
    assert_eq!(found.y, 2.0);
}
