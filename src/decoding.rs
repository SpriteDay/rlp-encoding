use crate::{
    RlpError,
    constants::{
        LONG_LIST_BASE_PREFIX, LONG_STRING_BASE_PREFIX, SHORT_LIST_BASE_PREFIX,
        SHORT_STRING_BASE_PREFIX,
    },
    types::RlpItem,
};

/// Decodes an RLP-encoded byte slice into an [`RlpItem`].
///
/// ```
/// use rlp_encoding::{decode, RlpItem};
///
/// let decoded = decode(&[0xC8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g']).unwrap();
/// assert_eq!(decoded, RlpItem::List(vec![
///     RlpItem::Bytes(b"cat".to_vec()),
///     RlpItem::Bytes(b"dog".to_vec()),
/// ]));
/// ```
pub fn decode(data: &[u8]) -> Result<RlpItem, RlpError> {
    decode_inner(data).map(|(item, _)| item)
}

fn decode_inner(data: &[u8]) -> Result<(RlpItem, usize), RlpError> {
    use RlpItem::{Bytes, List};

    let first_byte = data.first().ok_or(RlpError::EmptyInput)?;

    if *first_byte < SHORT_STRING_BASE_PREFIX {
        return Ok((Bytes(vec![*first_byte]), 1));
    }
    if *first_byte < LONG_STRING_BASE_PREFIX {
        let string_len = (first_byte - SHORT_STRING_BASE_PREFIX) as usize;
        let decoded = data.get(1..1 + string_len).ok_or(RlpError::UnexpectedEnd {
            expected: 1 + string_len,
            got: data.len(),
        })?;
        return Ok((Bytes(decoded.to_vec()), 1 + string_len));
    }
    if *first_byte < SHORT_LIST_BASE_PREFIX {
        let len_len = (first_byte - LONG_STRING_BASE_PREFIX) as usize;
        let string_len_bytes = data.get(1..1 + len_len).ok_or(RlpError::UnexpectedEnd {
            expected: 1 + len_len,
            got: data.len(),
        })?;
        let string_len = restore_integer(string_len_bytes);
        let decoded =
            data.get(1 + len_len..1 + len_len + string_len)
                .ok_or(RlpError::UnexpectedEnd {
                    expected: 1 + len_len + string_len,
                    got: data.len(),
                })?;
        return Ok((Bytes(decoded.to_vec()), 1 + len_len + string_len));
    }
    if *first_byte < LONG_LIST_BASE_PREFIX {
        let list_len = (first_byte - SHORT_LIST_BASE_PREFIX) as usize;
        let list = data.get(1..1 + list_len).ok_or(RlpError::UnexpectedEnd {
            expected: 1 + list_len,
            got: data.len(),
        })?;
        let mut consumed: usize = 0;
        let mut items = vec![];
        while consumed < list_len {
            let (item, n) = decode_inner(&list[consumed..])?;
            items.push(item);
            consumed += n;
        }
        return Ok((List(items), 1 + list_len));
    }

    let len_len = (first_byte - LONG_LIST_BASE_PREFIX) as usize;
    let list_len_bytes = data.get(1..1 + len_len).ok_or(RlpError::UnexpectedEnd {
        expected: 1 + len_len,
        got: data.len(),
    })?;
    let list_len = restore_integer(list_len_bytes);
    let list = data
        .get(1 + len_len..1 + len_len + list_len)
        .ok_or(RlpError::UnexpectedEnd {
            expected: 1 + len_len + list_len,
            got: data.len(),
        })?;
    let mut consumed: usize = 0;
    let mut items = vec![];
    while consumed < list_len {
        let (item, n) = decode_inner(&list[consumed..])?;
        items.push(item);
        consumed += n;
    }
    Ok((List(items), 1 + len_len + list_len))
}

fn restore_integer(data: &[u8]) -> usize {
    data.iter().fold(0, |acc, &b| (acc << 8) | b as usize)
}

#[cfg(test)]
mod tests {
    use crate::encode;

    use super::RlpItem::{Bytes, List};
    use super::*;
    // Decoding test
    #[test]
    fn decode_encoded() {
        let data = List(vec![
            Bytes("cat".as_bytes().to_vec()),
            Bytes("dog".as_bytes().to_vec()),
        ]);
        let encoded = encode(&data);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn decode_empty_string() {
        let decoded = decode(&[0x80]).unwrap();
        assert_eq!(decoded, Bytes(vec![]));
    }

    #[test]
    fn roundtrip_single_byte_zero() {
        let original = Bytes(vec![0x00]);
        assert_eq!(decode(&encode(&original)).unwrap(), original);
    }

    #[test]
    fn roundtrip_single_byte_0x80() {
        let original = Bytes(vec![0x80]);
        assert_eq!(decode(&encode(&original)).unwrap(), original);
    }

    #[test]
    fn decode_empty_list() {
        let decoded = decode(&[0xC0]).unwrap();
        assert_eq!(decoded, List(vec![]));
    }

    // --- Nested list ---
    #[test]
    fn roundtrip_nested_list() {
        // [ [], [[]], [ [], [[]] ] ] — the classic RLP spec example
        let data = List(vec![
            List(vec![]),
            List(vec![List(vec![])]),
            List(vec![List(vec![]), List(vec![List(vec![])])]),
        ]);
        let encoded = encode(&data);
        // Expected from the RLP spec
        assert_eq!(
            encoded,
            vec![0xC7, 0xC0, 0xC1, 0xC0, 0xC3, 0xC0, 0xC1, 0xC0]
        );
        assert_eq!(decode(&encoded).unwrap(), data);
    }

    // --- Long list (payload > 55 bytes) ---
    #[test]
    fn roundtrip_long_list() {
        // Build a list whose encoded payload exceeds 55 bytes
        let items: Vec<RlpItem> = (0..20).map(|_| Bytes("abc".as_bytes().to_vec())).collect();
        let data = List(items);
        let encoded = encode(&data);
        // Payload: 20 items * 4 bytes each ("abc" => [0x83, 0x61, 0x62, 0x63]) = 80 bytes
        // 80 > 55, so this is a long list: 0xF7 + 1, then 0x50 (80), then payload
        assert_eq!(encoded[0], 0xF8);
        assert_eq!(encoded[1], 80);
        assert_eq!(decode(&encoded).unwrap(), data);
    }

    // --- Roundtrip: list with mixed content ---
    #[test]
    fn roundtrip_mixed_list() {
        let data = List(vec![
            Bytes(vec![]),                        // empty string
            Bytes(vec![0x42]),                    // single byte
            Bytes(vec![0x80]),                    // single byte >= 0x80
            Bytes(b"hello".to_vec()),             // short string
            List(vec![Bytes(b"inner".to_vec())]), // nested list
        ]);
        assert_eq!(decode(&encode(&data)).unwrap(), data);
    }
}
