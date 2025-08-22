use super::connections::{Connection, ConnectionError, DataType, PortType};

#[test]
fn connects_matching_ports() {
    let conn = Connection::new(
        (0, 0, PortType::Out, DataType::Number),
        (1, 0, PortType::In, DataType::Number),
    );
    assert!(conn.is_ok());
}

#[test]
fn fails_on_mismatched_data() {
    let conn = Connection::new(
        (0, 0, PortType::Out, DataType::Number),
        (1, 0, PortType::In, DataType::Text),
    );
    assert_eq!(
        conn,
        Err(ConnectionError::DataTypeMismatch(
            DataType::Number,
            DataType::Text,
        ))
    );
}

#[test]
fn fails_on_wrong_port_direction() {
    let conn = Connection::new(
        (0, 0, PortType::In, DataType::Number),
        (1, 0, PortType::In, DataType::Number),
    );
    assert_eq!(
        conn,
        Err(ConnectionError::PortMismatch(PortType::In, PortType::In))
    );
}
