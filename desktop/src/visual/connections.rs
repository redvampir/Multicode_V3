use serde::{Deserialize, Serialize};

/// Direction of a block port.
///
/// A port either receives data (`In`) or sends it (`Out`).
///
/// # Examples
/// ```
/// use desktop::visual::connections::PortType;
/// let input = PortType::In;
/// let output = PortType::Out;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortType {
    In,
    Out,
}

/// The kind of data transported over a connection between blocks.
///
/// # Examples
/// ```
/// use desktop::visual::connections::DataType;
/// let ty = DataType::Boolean;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    Any,
    Number,
    Boolean,
    Text,
}

impl Default for DataType {
    fn default() -> Self {
        DataType::Any
    }
}

/// Connection between two block ports.
///
/// Both ports are identified by `(block_index, port_index)`.
///
/// # Examples
/// ```
/// use desktop::visual::connections::{Connection, DataType};
/// let conn = Connection { from: (0, 0), to: (1, 1), data_type: DataType::Number };
/// assert_eq!(conn.data_type, DataType::Number);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Connection {
    pub from: (usize, usize),
    pub to: (usize, usize),
    pub data_type: DataType,
}
