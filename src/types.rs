/// Represents an RLP-encoded item — either raw bytes or a nested list of items.
///
/// ```
/// use rlp_encoding::RlpItem;
///
/// let single = RlpItem::Bytes(vec![0x42]);
/// let list = RlpItem::List(vec![
///     RlpItem::Bytes(b"cat".to_vec()),
///     RlpItem::Bytes(b"dog".to_vec()),
/// ]);
/// ```
#[derive(Debug, PartialEq)]
pub enum RlpItem {
    Bytes(Vec<u8>),
    List(Vec<RlpItem>),
}

#[derive(Debug)]
pub enum RlpError {
    InvalidPrefix(u8),
    UnexpectedEnd { expected: usize, got: usize },
    EmptyInput,
    TrailingBytes { consumed: usize, total: usize },
}

impl std::fmt::Display for RlpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPrefix(b) => write!(f, "Invalid RLP prefix: 0x{b:02X}"),
            Self::UnexpectedEnd { expected, got } => {
                write!(f, "Expected {expected} bytes, got: {got}")
            }
            Self::EmptyInput => write!(f, "Got empty input"),
            Self::TrailingBytes { consumed, total } => write!(
                f,
                "Got trailing bytes, consumed: {consumed}, total: {total}"
            ),
        }
    }
}

impl std::error::Error for RlpError {}
