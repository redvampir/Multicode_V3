use backend::plugins::{Plugin, BlockDescriptor};

pub struct MyPlugin;

impl Plugin for MyPlugin {
    fn name(&self) -> &'static str {
        "my-plugin"
    }

    fn blocks(&self) -> Vec<BlockDescriptor> {
        vec![BlockDescriptor {
            kind: "MyBlock".to_string(),
            label: Some("Мой блок".to_string()),
        }]
    }
}
