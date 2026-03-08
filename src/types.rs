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
