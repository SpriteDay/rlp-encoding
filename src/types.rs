/// Represents an RLP-encoded item - either bytes (`RlptItem::Bytes`) or a nested list of items (`RlpItem::List`).
/// Example:
/// ```
/// use rlp_encoding::RlpItem::{List, Bytes}
/// let list = List(vec![Bytes([vec![0x12]]), Bytes([vec![0x12]])])
/// ```
#[derive(Debug, PartialEq)]
pub enum RlpItem {
    Bytes(Vec<u8>),
    List(Vec<RlpItem>),
}
