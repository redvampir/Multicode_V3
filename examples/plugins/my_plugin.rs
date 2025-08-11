use backend::plugins::{Plugin, BlockDescriptor};
use backend::meta::VisualMeta;
use chrono::Utc;
use std::collections::HashMap;
use serde_json::json;

pub struct MyPlugin;

impl Plugin for MyPlugin {
    fn name(&self) -> &'static str {
        "my-plugin"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn blocks(&self) -> Vec<BlockDescriptor> {
        vec![BlockDescriptor {
            kind: "MyBlock".to_string(),
            label: Some("Мой блок".to_string()),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }]
    }
}

// Example of constructing `VisualMeta` with plugin-specific extras.
#[allow(dead_code)]
fn example_meta_with_extras() -> VisualMeta {
    VisualMeta {
        id: "1".into(),
        x: 0.0,
        y: 0.0,
        tags: vec![],
        origin: None,
        translations: HashMap::new(),
        ai: None,
        extras: Some(json!({"outline": "blue"})),
        updated_at: Utc::now(),
    }
}
