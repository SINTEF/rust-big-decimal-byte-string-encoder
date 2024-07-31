# big-decimal-byte-string-encoder

[![Crates.io](https://img.shields.io/crates/v/big-decimal-byte-string-encoder.svg)](https://crates.io/crates/big-decimal-byte-string-encoder)
[![Documentation](https://docs.rs/big-decimal-byte-string-encoder/badge.svg)](https://docs.rs/big-decimal-byte-string-encoder)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

A Rust implementation of Google BigQuery's BigDecimalByteStringEncoder for the NUMERIC data type.

This crate provides functionality to encode and decode BigDecimal values to and from byte strings compatible with BigQuery's NUMERIC type, as used in the BigQuery Write API.

It goes nicely with [gcp-bigquery-client](https://github.com/lquerel/gcp-bigquery-client).

## Features

- Encode `BigDecimal` values to BigQuery NUMERIC bytes.
- Decode BigQuery NUMERIC bytes back to `BigDecimal` values.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
big-decimal-byte-string-encoder = "0.1.0"
```

## Usage

```rust
use bigdecimal::BigDecimal;
use big_decimal_byte_string_encoder::{decode_bigquery_bytes_to_bigdecimal, encode_bigdecimal_to_bigquery_bytes};
use std::str::FromStr;

let decimal = BigDecimal::from_str("123.456").unwrap();
let encoded = encode_bigdecimal_to_bigquery_bytes(&decimal).unwrap();
let decoded = decode_bigquery_bytes_to_bigdecimal(&encoded).unwrap();
assert_eq!(decimal, decoded);
```

## API Documentation

For detailed API documentation, please visit [docs.rs](https://docs.rs/big-decimal-byte-string-encoder).

## License

This project is licensed under the Apache License, Version 2.0. See the LICENSE file for details.

## Acknowledgments

This implementation is inspired by and ported from Google's BigQuery Write API. For more information, see the [BigQuery Write API documentation](https://cloud.google.com/bigquery/docs/write-api#data_type_conversions).

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.