# RLP Encoding
From-scratch implementation of [EVM RLP](https://ethereum.org/developers/docs/data-structures-and-encoding/rlp/)

## What is RLP?
Recursive Length Prefix (RLP) serialization used extensively in Ethereum's execution clients. Purpose of RLP is to encode arbitrarily nested arrays of binary data, and RLP is the primary encoding method used to serialize objects in Ethereum's execution layer

## Why from scratch?
I was practicing Rust and was doing deep dive into how EVM works, thought it would be a nice experience

## Usage
Example of RLP encoding of transaction:
```rust
use rlp_encoding::{encode, trim_integer};
use rlp_encoding::RlpItem::{Bytes, List};

let encoded = encode(&List(vec![
    Bytes(trim_integer(chain_id)),
    Bytes(trim_integer(nonce)),
    Bytes(trim_integer(max_priority_fee_per_gas)),
    Bytes(trim_integer(max_fee_per_gas)),
    Bytes(trim_integer(gas_limit)),
    Bytes(decode_hex_str(&recipient.as_str()).unwrap()),
    Bytes(trim_integer(value)),
    Bytes(data),
    List(
        access_list
            .iter()
            .map(|el| Bytes(decode_hex_str(&el.as_str()).unwrap()))
            .collect::<Vec<_>>(),
    ),
    Bytes(trim_integer(v)),
    Bytes(r),
    Bytes(s),
]));
```