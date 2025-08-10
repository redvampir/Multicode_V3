use backend::plugins::{Plugin, BlockDescriptor};

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
