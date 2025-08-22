use serde::{Deserialize, Serialize};
use std::fmt;

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

/// Errors that can occur when creating a [`Connection`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionError {
    /// The ports have incompatible directions.
    PortMismatch,
    /// The ports carry incompatible data types.
    DataTypeMismatch,
}

impl fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConnectionError::PortMismatch => write!(f, "incompatible port types"),
            ConnectionError::DataTypeMismatch => write!(f, "data type mismatch"),
        }
    }
}

impl std::error::Error for ConnectionError {}

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

impl Connection {
    /// Create a new [`Connection`] between two ports.
    ///
    /// `from` and `to` are tuples containing `(block_index, port_index, port_type, data_type)`.
    /// Returns an error if the ports have incompatible directions or data types.
    pub fn new(
        from: (usize, usize, PortType, DataType),
        to: (usize, usize, PortType, DataType),
    ) -> Result<Self, ConnectionError> {
        if from.2 != PortType::Out || to.2 != PortType::In {
            return Err(ConnectionError::PortMismatch);
        }

        let data_type = if from.3 == to.3 {
            from.3
        } else if from.3 == DataType::Any {
            to.3
        } else if to.3 == DataType::Any {
            from.3
        } else {
            return Err(ConnectionError::DataTypeMismatch);
        };

        Ok(Connection {
            from: (from.0, from.1),
            to: (to.0, to.1),
            data_type,
        })
    }
}
