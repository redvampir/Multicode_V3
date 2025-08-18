fn main() {
    if std::env::var("SKIP_TAURI_BUILD").is_err() {
        tauri_build::build();
    }

    // Generate JSON schema for VisualMeta
    #[path = "src/meta/types.rs"]
    mod types;
    use schemars::schema_for;

    let schema = schema_for!(types::VisualMeta);
    let schema_json = serde_json::to_string_pretty(&schema).unwrap();

    // Write schema next to backend and to frontend for reuse
    std::fs::write("meta.schema.json", &schema_json).unwrap();
    let frontend = std::path::Path::new("../frontend/src/editor/meta.schema.json");
    if let Some(parent) = frontend.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(frontend, schema_json).unwrap();
}
