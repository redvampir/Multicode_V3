use super::{ConflictResolutionMode, ResolutionOption, SyncEngine, SyncMessage, SyncSettings};
use chrono::Utc;
use multicode_core::meta::{self, VisualMeta, DEFAULT_VERSION};
use multicode_core::parser::Lang;
use std::collections::HashMap;
use tracing_test::traced_test;

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
    let mut engine = SyncEngine::new(Lang::Rust, SyncSettings::default());
    let meta = make_meta("test", DEFAULT_VERSION);
    let code = meta::upsert("", &meta, false);
    let (ret_code, metas, _diag) = engine
        .handle(SyncMessage::TextChanged(code.clone(), Lang::Rust))
        .unwrap();
    assert_eq!(ret_code, code);
    assert_eq!(metas.len(), 1);
    assert_eq!(metas[0].id, "test");
    assert_eq!(engine.state().metas.len(), 1);
}

#[test]
fn handle_returns_references_without_cloning() {
    let mut engine = SyncEngine::new(Lang::Rust, SyncSettings::default());
    let meta = make_meta("r", DEFAULT_VERSION);
    let code = meta::upsert("", &meta, false);
    let (ret_code, metas, _diag) = engine
        .handle(SyncMessage::TextChanged(code, Lang::Rust))
        .unwrap();
    let code_ptr = ret_code.as_ptr();
    let ids: Vec<String> = metas.iter().map(|m| m.id.clone()).collect();
    let _ = metas;
    let _ = ret_code;
    assert!(std::ptr::eq(code_ptr, engine.state().code.as_ptr()));
    assert!(ids
        .into_iter()
        .all(|id| engine.state().metas.contains_key(&id)));
}

#[test]
fn visual_changed_updates_state_code() {
    let mut engine = SyncEngine::new(Lang::Rust, SyncSettings::default());
    let _ = engine.handle(SyncMessage::TextChanged(String::new(), Lang::Rust));
    let meta = make_meta("block", DEFAULT_VERSION);
    let (result, metas, _diag) = engine
        .handle(SyncMessage::VisualChanged(meta.clone()))
        .unwrap();
    let result_ptr = result.as_ptr();
    let metas_owned: Vec<_> = metas.to_vec();
    let _ = metas;
    let _ = result;
    assert!(engine.state().code.contains("@VISUAL_META"));
    assert!(engine.state().code.contains("\"id\":\"block\""));
    assert!(std::ptr::eq(result_ptr, engine.state().code.as_ptr()));
    assert_eq!(metas_owned.len(), 1);
    assert_eq!(metas_owned[0].id, "block");
    assert_eq!(engine.state().metas.len(), 1);
    assert!(engine.state().metas.get("block").is_some());
}

#[test]
fn visual_changed_does_not_duplicate_meta() {
    let mut engine = SyncEngine::new(Lang::Rust, SyncSettings::default());
    let meta = make_meta("block", DEFAULT_VERSION);
    let code = meta::upsert("", &meta, false);
    let _ = engine.handle(SyncMessage::TextChanged(code, Lang::Rust));

    let updated = make_meta("block", DEFAULT_VERSION + 1);
    let _ = engine.handle(SyncMessage::VisualChanged(updated));

    assert_eq!(engine.state().metas.len(), 1);
    assert_eq!(
        engine.state().metas.get("block").unwrap().version,
        DEFAULT_VERSION + 1
    );
}

#[test]
fn visual_changed_zeros_version_defaults_to_constant() {
    let mut engine = SyncEngine::new(Lang::Rust, SyncSettings::default());
    let _ = engine.handle(SyncMessage::TextChanged(String::new(), Lang::Rust));
    let meta = make_meta("zero", 0);
    let _ = engine.handle(SyncMessage::VisualChanged(meta));
    assert_eq!(
        engine.state().metas.get("zero").unwrap().version,
        DEFAULT_VERSION
    );
    assert!(engine
        .state()
        .code
        .contains(&format!("\"version\":{}", DEFAULT_VERSION)));
}

#[test]
fn process_changes_stores_ids() {
    let mut engine = SyncEngine::new(Lang::Rust, SyncSettings::default());
    engine.process_changes(vec!["t1".into()], vec!["v1".into(), "v2".into()]);
    assert_eq!(engine.last_text_changes(), &["t1".to_string()]);
    assert_eq!(
        engine.last_visual_changes(),
        &["v1".to_string(), "v2".to_string()]
    );
}

#[test]
fn text_changed_updates_syntax_tree() {
    let mut engine = SyncEngine::new(Lang::Rust, SyncSettings::default());
    let _ = engine.handle(SyncMessage::TextChanged("fn main() {}".into(), Lang::Rust));
    assert!(!engine.state().syntax.nodes.is_empty());
}

#[test]
fn element_mapper_maps_ids_and_ranges() {
    let mut engine = SyncEngine::new(Lang::Rust, SyncSettings::default());
    let meta = make_meta("0", DEFAULT_VERSION);
    let code = meta::upsert("fn main() {}", &meta, false);
    let _ = engine.handle(SyncMessage::TextChanged(code, Lang::Rust));
    let range = engine.range_of("0").expect("range");
    assert_eq!(engine.id_at(range.start), Some("0"));
}

#[test]
fn id_and_range_handle_multiple_blocks() {
    let mut engine = SyncEngine::new(Lang::Rust, SyncSettings::default());
    let code = "fn a() {}\nfn b() {}\n";
    let _ = engine.handle(SyncMessage::TextChanged(code.into(), Lang::Rust));
    let ids: Vec<String> = engine
        .state()
        .syntax
        .nodes
        .iter()
        .filter(|n| n.block.kind == "Function/Define")
        .take(2)
        .map(|n| n.block.visual_id.clone())
        .collect();
    let mut code_with_metas = code.to_string();
    for id in &ids {
        code_with_metas = meta::upsert(&code_with_metas, &make_meta(id, DEFAULT_VERSION), false);
    }
    let _ = engine.handle(SyncMessage::TextChanged(code_with_metas, Lang::Rust));
    for id in ids {
        let range = engine.range_of(&id).expect("range");
        assert_eq!(engine.id_at(range.start), Some(id.as_str()));
        assert_eq!(engine.id_at(range.end.saturating_sub(1)), Some(id.as_str()));
    }
}

#[test]
fn id_at_position_finds_id_by_coordinates() {
    let mut engine = SyncEngine::new(Lang::Rust, SyncSettings::default());
    let meta = make_meta("0", DEFAULT_VERSION);
    let code = meta::upsert("fn main() {}\n", &meta, false);
    let _ = engine.handle(SyncMessage::TextChanged(code, Lang::Rust));
    assert_eq!(engine.id_at_position(0, 0), Some("0"));
    assert_eq!(engine.id_at_position(10, 0), None);
}

#[test]
fn conflict_resolver_applies_strategies() {
    let mut engine = SyncEngine::new(Lang::Rust, SyncSettings::default());
    let mut base = make_meta("c", DEFAULT_VERSION);
    base.translations
        .insert("rust".into(), "fn main() {}".into());
    let code = meta::upsert("", &base, false);
    let _ = engine.handle(SyncMessage::TextChanged(code, Lang::Rust));

    let mut visual = base.clone();
    visual.version = base.version + 1;
    visual.x = 10.0;
    visual.tags.push("v".into());
    let _ = engine.handle(SyncMessage::VisualChanged(visual));

    let resolved = engine.state().metas.get("c").unwrap();
    assert_eq!(resolved.x, 10.0); // movement prefers visual
    assert!(resolved.tags.contains(&"v".to_string())); // meta merge
    assert_eq!(resolved.translations.get("rust").unwrap(), "fn main() {}");
}

#[test]
fn reset_sync_clears_state() {
    let mut engine = SyncEngine::new(Lang::Rust, SyncSettings::default());
    let meta = make_meta("x", DEFAULT_VERSION);
    let code = meta::upsert("", &meta, false);
    let _ = engine.handle(SyncMessage::TextChanged(code, Lang::Rust));
    assert!(!engine.state().metas.is_empty());
    let _ = engine.handle(SyncMessage::ResetSync);
    assert!(engine.state().metas.is_empty());
    assert!(engine.state().code.is_empty());
}

#[test]
fn apply_resolution_overrides_policy_choice() {
    let mut engine = SyncEngine::new(
        Lang::Rust,
        SyncSettings {
            conflict_resolution: ConflictResolutionMode::PreferVisual,
            ..SyncSettings::default()
        },
    );
    let base = make_meta("c", DEFAULT_VERSION);
    let code = meta::upsert("fn main() {}", &base, false);
    let _ = engine.handle(SyncMessage::TextChanged(code, Lang::Rust));

    let mut visual = base.clone();
    visual.version += 1;
    visual.x = 10.0;
    let _ = engine.handle(SyncMessage::VisualChanged(visual));

    let mut text = base.clone();
    text.version += 2;
    text.x = 20.0;
    let code = meta::upsert("fn main() {}", &text, false);
    let _ = engine.handle(SyncMessage::TextChanged(code, Lang::Rust));
    assert_eq!(engine.state().metas.get("c").unwrap().x, 10.0);
    assert_eq!(engine.last_conflicts().len(), 1);

    engine.apply_resolution("c", ResolutionOption::Text);

    assert_eq!(engine.state().metas.get("c").unwrap().x, 20.0);
    assert!(engine.last_conflicts().is_empty());
    let updated = meta::read_all(&engine.state().code)
        .into_iter()
        .find(|m| m.id == "c")
        .unwrap();
    assert_eq!(updated.x, 20.0);
}

#[test]
fn text_changed_identifies_orphaned_blocks() {
    let mut engine = SyncEngine::new(Lang::Rust, SyncSettings::default());
    let mapped = make_meta("0", DEFAULT_VERSION);
    let orphan = make_meta("orphan", DEFAULT_VERSION);
    let code = meta::upsert("fn main() {}", &mapped, false);
    let code = meta::upsert(&code, &orphan, false);
    let _ = engine.handle(SyncMessage::TextChanged(code, Lang::Rust));
    assert_eq!(
        engine.last_diagnostics().orphaned_blocks,
        &["orphan".to_string()]
    );
}

#[test]
fn text_changed_reports_unmapped_code() {
    let mut engine = SyncEngine::new(Lang::Rust, SyncSettings::default());
    let root = make_meta("0", DEFAULT_VERSION);
    let code = meta::upsert("fn a() {}\nfn b() {}", &root, false);
    let _ = engine.handle(SyncMessage::TextChanged(code.clone(), Lang::Rust));
    let offset = code.find("fn b()").expect("offset of second function");
    assert!(engine
        .last_diagnostics()
        .unmapped_code
        .iter()
        .any(|r| offset >= r.start && offset < r.end));
}

#[test]
fn unmapped_code_has_exact_second_function_range() {
    let mut engine = SyncEngine::new(Lang::Rust, SyncSettings::default());
    let code = "fn a() {}\nfn b() {}\n";
    let _ = engine.handle(SyncMessage::TextChanged(code.into(), Lang::Rust));
    let mut fns = engine
        .state()
        .syntax
        .nodes
        .iter()
        .filter(|n| n.block.kind == "Function/Define");
    let first = fns.next().expect("first function");
    let second = fns.next().expect("second function");
    let first_id = first.block.visual_id.clone();
    let second_id = second.block.visual_id.clone();
    let code_with_meta = meta::upsert(code, &make_meta(&first_id, DEFAULT_VERSION), false);
    let _ = engine.handle(SyncMessage::TextChanged(code_with_meta, Lang::Rust));
    let second_range = engine
        .state()
        .syntax
        .nodes
        .iter()
        .find(|n| n.block.visual_id == second_id)
        .map(|n| n.block.range.clone())
        .expect("second function");
    let unmapped = &engine.last_diagnostics().unmapped_code;
    assert_eq!(unmapped.len(), 1);
    assert!(unmapped[0].start <= second_range.start && unmapped[0].end >= second_range.end);
}

#[test]
#[traced_test]
fn logs_warnings_for_mapping_issues() {
    let mut engine = SyncEngine::new(Lang::Rust, SyncSettings::default());
    let mapped = make_meta("0", DEFAULT_VERSION);
    let orphan = make_meta("orphan", DEFAULT_VERSION);
    let code = meta::upsert("fn a() {}\nfn b() {}", &mapped, false);
    let code = meta::upsert(&code, &orphan, false);
    let _ = engine.handle(SyncMessage::TextChanged(code, Lang::Rust));
    assert!(logs_contain("Orphaned metadata blocks"));
    assert!(logs_contain("orphan"));
    assert!(logs_contain("Unmapped code ranges"));
}

#[test]
fn visual_changed_respects_formatting_setting() {
    let base = make_meta("fmt", DEFAULT_VERSION);
    let json = serde_json::to_string(&base).unwrap();
    let code = format!("    <!-- @VISUAL_META {json} -->\nfn main() {{}}\n");

    // Formatting not preserved
    let mut engine = SyncEngine::new(
        Lang::Rust,
        SyncSettings {
            preserve_meta_formatting: false,
            ..SyncSettings::default()
        },
    );
    let _ = engine.handle(SyncMessage::TextChanged(code.clone(), Lang::Rust));
    let mut updated = base.clone();
    updated.x = 42.0;
    let _ = engine.handle(SyncMessage::VisualChanged(updated));
    assert!(engine.state().code.starts_with("<!-- @VISUAL_META"));

    // Formatting preserved
    let mut engine = SyncEngine::new(
        Lang::Rust,
        SyncSettings {
            preserve_meta_formatting: true,
            ..SyncSettings::default()
        },
    );
    let _ = engine.handle(SyncMessage::TextChanged(code, Lang::Rust));
    let mut updated = base.clone();
    updated.x = 24.0;
    let _ = engine.handle(SyncMessage::VisualChanged(updated));
    assert!(engine.state().code.starts_with("    <!-- @VISUAL_META"));
}

#[test]
fn apply_resolution_respects_formatting_setting() {
    let base = make_meta("fmt", DEFAULT_VERSION);
    let json = serde_json::to_string(&base).unwrap();
    let code = format!("    <!-- @VISUAL_META {json} -->\nfn main() {{}}\n");

    // Formatting not preserved
    let mut engine = SyncEngine::new(
        Lang::Rust,
        SyncSettings {
            preserve_meta_formatting: false,
            ..SyncSettings::default()
        },
    );
    let _ = engine.handle(SyncMessage::TextChanged(code.clone(), Lang::Rust));
    engine.apply_resolution("fmt", ResolutionOption::Visual);
    assert!(engine.state().code.starts_with("<!-- @VISUAL_META"));

    // Formatting preserved
    let mut engine = SyncEngine::new(
        Lang::Rust,
        SyncSettings {
            preserve_meta_formatting: true,
            ..SyncSettings::default()
        },
    );
    let _ = engine.handle(SyncMessage::TextChanged(code, Lang::Rust));
    engine.apply_resolution("fmt", ResolutionOption::Visual);
    assert!(engine.state().code.starts_with("    <!-- @VISUAL_META"));
}

mod engine {
    use super::*;

    #[test]
    fn malformed_meta() {
        let mut engine = SyncEngine::new(Lang::Rust, SyncSettings::default());
        let code = "// @VISUAL_META {\"id\":\"oops\",}\nfn main() {}\n";
        let (_code, _metas, diag) = engine
            .handle(SyncMessage::TextChanged(code.into(), Lang::Rust))
            .unwrap();
        assert!(!diag.unmapped_code.is_empty());
    }
}
