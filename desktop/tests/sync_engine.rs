use desktop::sync::{SyncEngine, SyncMessage, SyncSettings};
use multicode_core::meta::{self, VisualMeta, DEFAULT_VERSION};
use multicode_core::parser::Lang;
use chrono::Utc;
use std::collections::HashMap;
use std::time::Instant;

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
fn text_edit_updates_visual_meta() {
    let mut engine = SyncEngine::new(Lang::Rust, SyncSettings::default());

    // Initial code with meta inserted.
    let original_meta = make_meta("block", DEFAULT_VERSION);
    let mut code = meta::upsert("fn main() {}\n", &original_meta, false);
    let _ = engine
        .handle(SyncMessage::TextChanged(code.clone(), Lang::Rust))
        .expect("text change");

    assert_eq!(engine.state().metas.get("block").unwrap().x, 1.0);

    // Simulate manual text edit updating @VISUAL_META.
    let mut metas = meta::read_all(&code);
    metas[0].x = 42.0;
    code = meta::upsert(&code, &metas[0], false);

    let _ = engine
        .handle(SyncMessage::TextChanged(code.clone(), Lang::Rust))
        .expect("text change");

    // SyncEngine should reflect the updated meta.
    let metas_from_state = meta::read_all(&engine.state().code);
    assert_eq!(metas_from_state[0].x, 42.0);
    assert_eq!(engine.state().metas.get("block").unwrap().x, 42.0);
}

#[test]
fn visual_block_change_updates_text() {
    let mut engine = SyncEngine::new(Lang::Rust, SyncSettings::default());
    let _ = engine.handle(SyncMessage::TextChanged(String::new(), Lang::Rust));

    let meta = make_meta("v", DEFAULT_VERSION);
    let metas_owned: Vec<_> = {
        let (_code, metas, _diag) = engine
            .handle(SyncMessage::VisualChanged(meta.clone()))
            .expect("visual change");
        metas.to_vec()
    };

    // Engine should add meta into its text representation.
    let metas_from_code = meta::read_all(&engine.state().code);
    assert_eq!(metas_from_code.len(), 1);
    assert_eq!(metas_from_code[0].id, "v");
    assert_eq!(metas_owned[0].id, "v");
}

#[test]
fn text_meta_deletion_removes_state() {
    let mut engine = SyncEngine::new(Lang::Rust, SyncSettings::default());

    let meta = make_meta("block", DEFAULT_VERSION);
    let mut code = meta::upsert("fn main() {}\n", &meta, false);

    let _ = engine
        .handle(SyncMessage::TextChanged(code.clone(), Lang::Rust))
        .expect("text change");

    assert!(engine.state().metas.contains_key("block"));

    code = meta::remove_all(&code);

    let _ = engine
        .handle(SyncMessage::TextChanged(code.clone(), Lang::Rust))
        .expect("text change");

    assert!(!engine.state().metas.contains_key("block"));
}

#[test]
fn sync_large_file_performance() {
    let mut engine = SyncEngine::new(Lang::Rust, SyncSettings::default());
    let mut code = String::new();

    for i in 0..1000 {
        let meta = make_meta(&format!("id{i}"), DEFAULT_VERSION);
        let snippet = format!("fn f{i}() {{}}\n");
        let snippet = meta::upsert(&snippet, &meta, false);
        code.push_str(&snippet);
    }

    let start = Instant::now();
    let _ = engine
        .handle(SyncMessage::TextChanged(code, Lang::Rust))
        .expect("large text change");
    let elapsed = start.elapsed();

    // Ensure syncing large files remains reasonably fast.
    assert!(elapsed.as_secs() < 5, "syncing took {:?}", elapsed);
}

