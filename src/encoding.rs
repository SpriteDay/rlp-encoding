use num_traits::{PrimInt, ToBytes, Unsigned};

use crate::{
    constants::{
        LONG_LIST_BASE_PREFIX, LONG_STRING_BASE_PREFIX, SHORT_LIST_BASE_PREFIX,
        SHORT_STRING_BASE_PREFIX,
    },
    types::RlpItem::{self, Bytes, List},
};

/// Encodes an [`RlpItem`] into its RLP byte representation.
///
/// ```
/// use rlp_encoding::{encode, RlpItem};
///
/// let item = RlpItem::List(vec![
///     RlpItem::Bytes(b"cat".to_vec()),
///     RlpItem::Bytes(b"dog".to_vec()),
/// ]);
/// let encoded = encode(&item);
/// assert_eq!(encoded, vec![0xC8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g']);
/// ```
pub fn encode(item: &RlpItem) -> Vec<u8> {
    match item {
        Bytes(bytes) => convert_string(bytes),
        List(list) => convert_list(list),
    }
}

fn convert_list(input: &[RlpItem]) -> Vec<u8> {
    // First, encode all items anc collect into a single payload
    let payload: Vec<u8> = input.iter().flat_map(|item| encode(item)).collect();

    let len = payload.len();
    let long_list_threshold = LONG_LIST_BASE_PREFIX - SHORT_LIST_BASE_PREFIX;
    if len <= long_list_threshold as usize {
        let mut result = vec![SHORT_LIST_BASE_PREFIX + len as u8];
        result.extend_from_slice(&payload);
        return result;
    };
    let trimmed_len = trim_integer(len);
    let len_len = trimmed_len.len();
    let mut result = vec![LONG_LIST_BASE_PREFIX + len_len as u8];
    result.extend(trimmed_len);
    result.extend(payload);
    result
}

fn convert_string(input: &[u8]) -> Vec<u8> {
    let len = input.len();
    let long_string_threshold = LONG_STRING_BASE_PREFIX - SHORT_STRING_BASE_PREFIX;
    if len == 1 && input[0] < SHORT_STRING_BASE_PREFIX {
        return input.to_vec();
    };
    if len <= long_string_threshold as usize {
        let mut result = vec![SHORT_STRING_BASE_PREFIX + len as u8];
        result.extend_from_slice(input);
        return result;
    };
    let trimmed_len = trim_integer(len);
    let len_len = trimmed_len.len();
    let mut result = vec![LONG_STRING_BASE_PREFIX + len_len as u8];
    result.extend(trimmed_len);
    result.extend(input);
    result
}

/// Trims leading zero bytes from an unsigned integer's big-endian representation.
///
/// ```
/// use rlp_encoding::trim_integer;
///
/// assert_eq!(trim_integer(0_u32), vec![]);
/// assert_eq!(trim_integer(255_u32), vec![0xFF]);
/// assert_eq!(trim_integer(256_u32), vec![0x01, 0x00]);
/// ```
///
/// Only accepts unsigned integers:
/// ```compile_fail
/// use rlp_encoding::trim_integer;
/// trim_integer(-1_i32); // ~ the trait bound `i32: Unsigned` is not satisfied
/// ```
pub fn trim_integer<T: PrimInt + Unsigned + ToBytes>(num: T) -> Vec<u8> {
    let bytes = num.to_be_bytes();
    bytes
        .as_ref()
        .iter()
        .skip_while(|b| **b == 0)
        .copied()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::RlpItem::{Bytes, List};
    use super::*;

    // Encoding test
    #[test]
    fn encode_2d_array() {
        let data = List(vec![
            Bytes("cat".as_bytes().to_vec()),
            Bytes("dog".as_bytes().to_vec()),
        ]);
        let result = encode(&data);
        assert_eq!(
            result,
            vec![0xC8, 0x83, 0x63, 0x61, 0x74, 0x83, 0x64, 0x6F, 0x67]
        );
    }

    // String conversion tests
    #[test]
    fn convert_1_char() {
        let result = convert_string("B".as_bytes());
        assert_eq!(result, vec![0x42]);
    }

    #[test]
    fn convert_short_string() {
        let result = convert_string("dog".as_bytes());
        println!("result: {result:?}");
        assert_eq!(result, vec![0x83, 0x64, 0x6F, 0x67]);
    }

    #[test]
    fn convert_long_string() {
        let input = vec![0x61_u8; 56]; // 56 bytes of 'a'
        let result = convert_string(&input);
        // first byte should be 0xB7 + 1 = 0xB8 (length takes 1 byte)
        // second byte should be 56 = 0x38
        // then 56 bytes of 0x61
        println!("result: {result:?}");
        assert_eq!(result[0], 0xB8);
        assert_eq!(result[1], 0x38);
        assert_eq!(result.len(), 58);
    }

    // Integer trimming tests
    #[test]
    fn trim_zero_integer() {
        let result = trim_integer(0_u32);
        assert_eq!(result, vec![]);
    }

    #[test]
    fn trim_small_u8() {
        let result = trim_integer(42_u32);
        assert_eq!(result, vec![0x2A]);
    }

    #[test]
    fn trim_max_u8() {
        let result = trim_integer(255_u32);
        assert_eq!(result, vec![0xFF]);
    }

    // --- Empty string ---
    #[test]
    fn encode_empty_string() {
        // Empty string => [0x80]
        let result = convert_string(&[]);
        assert_eq!(result, vec![0x80]);
    }

    // --- Single byte edge cases ---
    #[test]
    fn encode_single_byte_zero() {
        // 0x00 is below 0x80, so it encodes as itself
        let result = convert_string(&[0x00]);
        assert_eq!(result, vec![0x00]);
    }

    #[test]
    fn encode_single_byte_0x7f() {
        // 0x7F is the largest single-byte value that encodes as itself
        let result = convert_string(&[0x7F]);
        assert_eq!(result, vec![0x7F]);
    }

    #[test]
    fn encode_single_byte_0x80() {
        // 0x80 is NOT below 0x80, so it needs a length prefix
        let result = convert_string(&[0x80]);
        assert_eq!(result, vec![0x81, 0x80]);
    }

    // --- Empty list ---
    #[test]
    fn encode_empty_list() {
        // Empty list => [0xC0]
        let result = encode(&List(vec![]));
        assert_eq!(result, vec![0xC0]);
    }

    // --- Short string at max boundary (55 bytes) ---
    #[test]
    fn encode_55_byte_string() {
        let input = vec![0x61_u8; 55]; // exactly 55 bytes — still short string
        let result = convert_string(&input);
        assert_eq!(result[0], 0x80 + 55); // 0xB7
        assert_eq!(result.len(), 56);
    }

    // --- Integer trimming: multi-byte ---
    #[test]
    fn trim_two_byte_integer() {
        let result = trim_integer(0x0100_u32);
        assert_eq!(result, vec![0x01, 0x00]);
    }
}
