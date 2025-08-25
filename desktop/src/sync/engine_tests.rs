use super::{SyncEngine, SyncMessage};
use chrono::Utc;
use multicode_core::meta::{self, VisualMeta, DEFAULT_VERSION};
use multicode_core::parser::Lang;
use std::collections::HashMap;

fn make_meta(id: &str, version: u32) -> VisualMeta {
    VisualMeta {
        version,
        id: id.to_string(),
        x: 1.0,
        y: 2.0,
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

#[test]
fn text_changed_returns_metas() {
    let mut engine = SyncEngine::new(Lang::Rust);
    let meta = make_meta("test", DEFAULT_VERSION);
    let code = meta::upsert("", &meta);
    let (ret_code, metas) = engine
        .handle(SyncMessage::TextChanged(code.clone(), Lang::Rust))
        .unwrap();
    assert_eq!(ret_code, code);
    assert_eq!(metas.len(), 1);
    assert_eq!(metas[0].id, "test");
    assert_eq!(engine.state().metas.len(), 1);
}

#[test]
fn visual_changed_updates_state_code() {
    let mut engine = SyncEngine::new(Lang::Rust);
    let _ = engine.handle(SyncMessage::TextChanged(String::new(), Lang::Rust));
    let meta = make_meta("block", DEFAULT_VERSION);
    let (result, metas) = engine
        .handle(SyncMessage::VisualChanged(meta.clone()))
        .unwrap();
    assert!(result.contains("@VISUAL_META"));
    assert!(result.contains("\"id\":\"block\""));
    assert_eq!(result, engine.state().code);
    assert_eq!(metas.len(), 1);
    assert_eq!(metas[0].id, "block");
    assert_eq!(engine.state().metas.len(), 1);
    assert!(engine.state().metas.get("block").is_some());
}

#[test]
fn visual_changed_does_not_duplicate_meta() {
    let mut engine = SyncEngine::new(Lang::Rust);
    let meta = make_meta("block", DEFAULT_VERSION);
    let code = meta::upsert("", &meta);
    let _ = engine.handle(SyncMessage::TextChanged(code, Lang::Rust));

    let updated = make_meta("block", DEFAULT_VERSION + 1);
    let _ = engine.handle(SyncMessage::VisualChanged(updated));

    assert_eq!(engine.state().metas.len(), 1);
    assert_eq!(
        engine
            .state()
            .metas
            .get("block")
            .unwrap()
            .version,
        DEFAULT_VERSION + 1
    );
}

#[test]
fn visual_changed_zeros_version_defaults_to_constant() {
    let mut engine = SyncEngine::new(Lang::Rust);
    let _ = engine.handle(SyncMessage::TextChanged(String::new(), Lang::Rust));
    let meta = make_meta("zero", 0);
    let _ = engine.handle(SyncMessage::VisualChanged(meta));
    assert_eq!(
        engine
            .state()
            .metas
            .get("zero")
            .unwrap()
            .version,
        DEFAULT_VERSION
    );
    assert!(engine
        .state()
        .code
        .contains(&format!("\"version\":{}", DEFAULT_VERSION)));
}

#[test]
fn process_changes_stores_ids() {
    let mut engine = SyncEngine::new(Lang::Rust);
    engine.process_changes(vec!["t1".into()], vec!["v1".into(), "v2".into()]);
    assert_eq!(engine.last_text_changes(), &["t1".to_string()]);
    assert_eq!(
        engine.last_visual_changes(),
        &["v1".to_string(), "v2".to_string()]
    );
}

#[test]
fn text_changed_updates_syntax_tree() {
    let mut engine = SyncEngine::new(Lang::Rust);
    let _ = engine.handle(SyncMessage::TextChanged("fn main() {}".into(), Lang::Rust));
    assert!(!engine.state().syntax.nodes.is_empty());
}

#[test]
fn element_mapper_maps_ids_and_ranges() {
    let mut engine = SyncEngine::new(Lang::Rust);
    let meta = make_meta("0", DEFAULT_VERSION);
    let code = meta::upsert("fn main() {}", &meta);
    let _ = engine.handle(SyncMessage::TextChanged(code, Lang::Rust));
    let range = engine.range_of("0").expect("range");
    assert_eq!(engine.id_at(range.start), Some("0"));
}
