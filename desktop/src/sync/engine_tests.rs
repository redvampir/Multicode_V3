use super::{SyncEngine, SyncMessage};
use chrono::Utc;
use multicode_core::meta::{self, VisualMeta, DEFAULT_VERSION};
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
    let mut engine = SyncEngine::new();
    let meta = make_meta("test", DEFAULT_VERSION);
    let code = meta::upsert("", &meta);
    let (ret_code, metas) = engine
        .handle(SyncMessage::TextChanged(code.clone()))
        .unwrap();
    assert_eq!(ret_code, code);
    assert_eq!(metas.len(), 1);
    assert_eq!(metas[0].id, "test");
    assert_eq!(engine.state().metas.len(), 1);
}

#[test]
fn visual_changed_updates_state_code() {
    let mut engine = SyncEngine::new();
    let _ = engine.handle(SyncMessage::TextChanged(String::new()));
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
    assert_eq!(engine.state().metas[0].id, "block");
}

#[test]
fn visual_changed_does_not_duplicate_meta() {
    let mut engine = SyncEngine::new();
    let meta = make_meta("block", DEFAULT_VERSION);
    let code = meta::upsert("", &meta);
    let _ = engine.handle(SyncMessage::TextChanged(code));

    let updated = make_meta("block", DEFAULT_VERSION + 1);
    let _ = engine.handle(SyncMessage::VisualChanged(updated));

    assert_eq!(engine.state().metas.len(), 1);
    assert_eq!(engine.state().metas[0].version, DEFAULT_VERSION + 1);
}

#[test]
fn visual_changed_zeros_version_defaults_to_constant() {
    let mut engine = SyncEngine::new();
    let _ = engine.handle(SyncMessage::TextChanged(String::new()));
    let meta = make_meta("zero", 0);
    let _ = engine.handle(SyncMessage::VisualChanged(meta));
    assert_eq!(engine.state().metas[0].version, DEFAULT_VERSION);
    assert!(engine
        .state()
        .code
        .contains(&format!("\"version\":{}", DEFAULT_VERSION)));
}
