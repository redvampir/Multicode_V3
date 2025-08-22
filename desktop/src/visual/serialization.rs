use crate::visual::canvas::{Connection, DataType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionMeta {
    pub from: (usize, usize),
    pub to: (usize, usize),
    #[serde(default)]
    pub data_type: DataType,
}

pub fn serialize_to_meta(connections: &[Connection]) -> Vec<ConnectionMeta> {
    connections
        .iter()
        .map(|c| ConnectionMeta {
            from: c.from,
            to: c.to,
            data_type: c.data_type,
        })
        .collect()
}

pub fn load_from_meta(meta: &[ConnectionMeta]) -> Vec<Connection> {
    meta.iter()
        .map(|m| Connection {
            from: m.from,
            to: m.to,
            data_type: m.data_type,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_preserves_type() {
        let connections = vec![Connection {
            from: (1, 0),
            to: (2, 1),
            data_type: DataType::Boolean,
        }];
        let meta = serialize_to_meta(&connections);
        let restored = load_from_meta(&meta);
        assert_eq!(restored[0].data_type, DataType::Boolean);
    }

    #[test]
    fn missing_type_defaults_to_any() {
        let json = "[{\"from\":[0,1],\"to\":[2,3]}]";
        let meta: Vec<ConnectionMeta> = serde_json::from_str(json).unwrap();
        let restored = load_from_meta(&meta);
        assert_eq!(restored[0].data_type, DataType::Any);
    }
}
