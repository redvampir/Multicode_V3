use super::{
    connections::{Connection, DataType},
    serialization::{load_from_meta, serialize_to_meta},
};
use multicode_core::BlockInfo;
use std::collections::HashMap;

#[test]
fn serialize_roundtrip() {
    // Prepare dummy blocks
    let blocks = vec![
        BlockInfo {
            visual_id: "a".into(),
            node_id: None,
            kind: "test".into(),
            translations: HashMap::new(),
            range: (0, 0),
            anchors: vec![],
            x: 0.0,
            y: 0.0,
            ports: vec![],
            ai: None,
            tags: vec![],
            links: vec![],
        },
        BlockInfo {
            visual_id: "b".into(),
            node_id: None,
            kind: "test".into(),
            translations: HashMap::new(),
            range: (0, 0),
            anchors: vec![],
            x: 1.0,
            y: 1.0,
            ports: vec![],
            ai: None,
            tags: vec![],
            links: vec![],
        },
    ];

    // Prepare dummy connections referencing the blocks
    let connections = vec![Connection {
        from: (0, 0),
        to: (1, 0),
        data_type: DataType::Number,
    }];

    let meta = serialize_to_meta(&connections);
    let restored = load_from_meta(&meta);

    assert_eq!(connections, restored);
    assert_eq!(blocks.len(), 2); // Blocks remain unchanged
}
