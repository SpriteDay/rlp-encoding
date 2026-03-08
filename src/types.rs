#[derive(Debug, PartialEq)]
pub enum RlpItem {
    Bytes(Vec<u8>),
    List(Vec<RlpItem>),
}
