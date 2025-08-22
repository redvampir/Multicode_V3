#[derive(Debug, PartialEq)]
enum PortDirection {
    In,
    Out,
}

#[derive(Debug, PartialEq)]
enum DataType {
    Integer,
    Text,
}

#[derive(Debug)]
struct Port {
    direction: PortDirection,
    data: DataType,
}

fn can_connect(from: &Port, to: &Port) -> Result<(), &'static str> {
    match (&from.direction, &to.direction) {
        (PortDirection::Out, PortDirection::In) => {
            if from.data == to.data {
                Ok(())
            } else {
                Err("data type mismatch")
            }
        }
        (PortDirection::Out, PortDirection::Out) | (PortDirection::In, PortDirection::In) |
        (PortDirection::In, PortDirection::Out) => Err("incompatible ports"),
    }
}

#[test]
fn connects_matching_ports() {
    let from = Port { direction: PortDirection::Out, data: DataType::Integer };
    let to = Port { direction: PortDirection::In, data: DataType::Integer };
    assert!(can_connect(&from, &to).is_ok());
}

#[test]
fn fails_on_mismatched_data() {
    let from = Port { direction: PortDirection::Out, data: DataType::Integer };
    let to = Port { direction: PortDirection::In, data: DataType::Text };
    assert!(can_connect(&from, &to).is_err());
}

#[test]
fn fails_on_wrong_port_direction() {
    let from = Port { direction: PortDirection::In, data: DataType::Integer };
    let to = Port { direction: PortDirection::In, data: DataType::Integer };
    assert!(can_connect(&from, &to).is_err());
}

