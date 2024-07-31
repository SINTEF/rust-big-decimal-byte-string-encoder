//! A Rust implementation of Google BigQuery's BigDecimalByteStringEncoder for the NUMERIC data type.
//!
//! This crate provides functionality to encode and decode BigDecimal values
//! to and from byte strings compatible with BigQuery's NUMERIC type, as used in the BigQuery Write API.
//!
//! # Examples
//!
//! ```
//! use bigdecimal::BigDecimal;
//! use big_decimal_byte_string_encoder::{decode_bigquery_bytes_to_bigdecimal, encode_bigdecimal_to_bigquery_bytes};
//! use std::str::FromStr;
//!
//! let decimal = BigDecimal::from_str("123.456").unwrap();
//! let encoded = encode_bigdecimal_to_bigquery_bytes(&decimal).unwrap();
//! let decoded = decode_bigquery_bytes_to_bigdecimal(&encoded).unwrap();
//! assert_eq!(decimal, decoded);
//! ```

use bigdecimal::BigDecimal;
use num_bigint::{BigInt, Sign};
use once_cell::sync::Lazy;
use std::str::FromStr;
use thiserror::Error;

/// The scale used for NUMERIC values in BigQuery.
const NUMERIC_SCALE: i64 = 9;

/// The maximum value for a NUMERIC type in BigQuery.
static MAX_NUMERIC_VALUE: Lazy<BigDecimal> =
    Lazy::new(|| BigDecimal::from_str("99999999999999999999999999999.999999999").unwrap());

/// The minimum value for a NUMERIC type in BigQuery.
static MIN_NUMERIC_VALUE: Lazy<BigDecimal> =
    Lazy::new(|| BigDecimal::from_str("-99999999999999999999999999999.999999999").unwrap());

/// Errors that can occur during encoding or decoding.
#[derive(Error, Debug)]
pub enum NumericEncoderError {
    #[error("Scale exceeds maximum: {0} (allowed: {1})")]
    ScaleExceeded(i64, i64),
    #[error("Numeric overflow: {0}")]
    Overflow(String),
}

fn to_java_byte_array(value: &BigInt) -> Vec<u8> {
    let (sign, mut bytes) = value.to_bytes_be();

    if sign == Sign::Minus {
        if bytes.is_empty() {
            bytes.push(0);
        }

        for byte in &mut bytes {
            *byte = !*byte;
        }

        let mut carry = true;
        for byte in bytes.iter_mut().rev() {
            if carry {
                if *byte == 0xFF {
                    *byte = 0;
                } else {
                    *byte += 1;
                    carry = false;
                }
            } else {
                break;
            }
        }

        if carry {
            bytes.insert(0, 1);
        }

        if bytes[0] & 0x80 == 0 {
            bytes.insert(0, 0xFF);
        }
    } else if !bytes.is_empty() && bytes[0] & 0x80 != 0 {
        bytes.insert(0, 0);
    }

    bytes
}

fn from_java_byte_array(bytes: &[u8]) -> BigInt {
    if bytes.is_empty() {
        return BigInt::from(0);
    }

    let is_negative = bytes[0] & 0x80 != 0;

    if is_negative {
        let mut complemented = Vec::with_capacity(bytes.len());
        let mut carry = true;

        for &byte in bytes.iter().rev() {
            let mut complemented_byte = !byte;
            if carry {
                if complemented_byte == 0xFF {
                    complemented_byte = 0;
                } else {
                    complemented_byte += 1;
                    carry = false;
                }
            }
            complemented.push(complemented_byte);
        }

        complemented.reverse();

        while complemented.len() > 1 && complemented[0] == 0xFF {
            complemented.remove(0);
        }

        BigInt::from_bytes_be(Sign::Minus, &complemented)
    } else {
        let mut start = 0;
        while start < bytes.len() - 1 && bytes[start] == 0 {
            start += 1;
        }

        BigInt::from_bytes_be(Sign::Plus, &bytes[start..])
    }
}

/// Encodes a BigDecimal value to a byte string compatible with BigQuery's NUMERIC type.
///
/// # Arguments
///
/// * `decimal` - The BigDecimal value to encode.
///
/// # Returns
///
/// A Result containing either the encoded byte string or a NumericEncoderError.
///
/// # Examples
///
/// ```
/// use bigdecimal::BigDecimal;
/// use std::str::FromStr;
/// use big_decimal_byte_string_encoder::encode_bigdecimal_to_bigquery_bytes;
///
/// let decimal = BigDecimal::from_str("123.456").unwrap();
/// let encoded = encode_bigdecimal_to_bigquery_bytes(&decimal).unwrap();
/// ```
pub fn encode_bigdecimal_to_bigquery_bytes(
    decimal: &BigDecimal,
) -> Result<Vec<u8>, NumericEncoderError> {
    let scale = decimal.fractional_digit_count();
    if !(0..=NUMERIC_SCALE).contains(&scale) {
        return Err(NumericEncoderError::ScaleExceeded(scale, NUMERIC_SCALE));
    }

    if decimal < &*MIN_NUMERIC_VALUE || decimal > &*MAX_NUMERIC_VALUE {
        return Err(NumericEncoderError::Overflow(decimal.to_string()));
    }

    let scaled = decimal.with_scale(NUMERIC_SCALE);
    let (scaled_value, _) = scaled.as_bigint_and_exponent();
    let mut bytes = to_java_byte_array(&scaled_value);
    bytes.reverse();
    Ok(bytes)
}

/// Decodes a byte string to a BigDecimal value.
///
/// # Arguments
///
/// * `bytes` - The byte string to decode.
///
/// # Returns
///
/// A Result containing either the decoded BigDecimal value or a NumericEncoderError.
///
/// # Examples
///
/// ```
/// use big_decimal_byte_string_encoder::decode_bigquery_bytes_to_bigdecimal;
///
/// let encoded = vec![0, 140, 134, 71];
/// let decoded = decode_bigquery_bytes_to_bigdecimal(&encoded).unwrap();
/// ```
pub fn decode_bigquery_bytes_to_bigdecimal(
    bytes: &[u8],
) -> Result<BigDecimal, NumericEncoderError> {
    let mut bytes = bytes.to_vec();
    bytes.reverse();

    let scaled_value = from_java_byte_array(&bytes);

    let decimal_value = BigDecimal::from((scaled_value, NUMERIC_SCALE));
    if decimal_value > *MAX_NUMERIC_VALUE || decimal_value < *MIN_NUMERIC_VALUE {
        return Err(NumericEncoderError::Overflow(decimal_value.to_string()));
    }

    Ok(decimal_value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bigdecimal::FromPrimitive;

    fn test_value(value: &str, binary: Vec<u8>) {
        let original = BigDecimal::from_str(value).unwrap();
        let encoded = encode_bigdecimal_to_bigquery_bytes(&original).unwrap();
        let mut reversed_binary = binary.clone();
        reversed_binary.reverse();
        assert_eq!(encoded, reversed_binary);
        let decoded = decode_bigquery_bytes_to_bigdecimal(&encoded).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_encode_decode() {
        test_value("0", vec![0]);
        test_value("1.2", vec![71, 134, 140, 0]);
        test_value("-1.2", vec![184, 121, 116, 0]);
        test_value(
            "99999999999999999999999999999.999999999",
            vec![
                75, 59, 76, 168, 90, 134, 196, 122, 9, 138, 34, 63, 255, 255, 255, 255,
            ],
        );
        test_value(
            "-99999999999999999999999999999.999999999",
            vec![
                180, 196, 179, 87, 165, 121, 59, 133, 246, 117, 221, 192, 0, 0, 0, 1,
            ],
        );
        test_value(
            "-123456789.42001",
            vec![254, 73, 100, 180, 65, 130, 149, 240],
        );
        test_value("12.345", vec![2, 223, 209, 192, 64]);
        test_value("1", vec![59, 154, 202, 0]);
        test_value("2", vec![119, 53, 148, 0]);
        test_value("-1", vec![196, 101, 54, 0]);
        test_value("128", vec![29, 205, 101, 0, 0]);
        test_value("-128", vec![226, 50, 155, 0, 0]);
        test_value("12702228", vec![45, 32, 155, 235, 203, 200, 0]);
    }

    #[test]
    fn test_encode_decode_random() {
        for _ in 0..1000 {
            let original = BigDecimal::from_f64(rand::random::<f64>()).unwrap();
            let scale = rand::random::<u32>() % 8 + 2;
            let original = original.with_scale(scale as i64);
            let encoded = encode_bigdecimal_to_bigquery_bytes(&original).unwrap();
            let decoded = decode_bigquery_bytes_to_bigdecimal(&encoded).unwrap();
            assert_eq!(original, decoded);
        }
    }

    #[test]
    fn test_overflow() {
        let too_big = BigDecimal::from_str("100000000000000000000000000000").unwrap();
        assert!(matches!(
            encode_bigdecimal_to_bigquery_bytes(&too_big),
            Err(NumericEncoderError::Overflow(_))
        ));
    }

    #[test]
    fn test_scale_exceeded() {
        let too_precise = BigDecimal::from_str("1.0000000001").unwrap();
        assert!(matches!(
            encode_bigdecimal_to_bigquery_bytes(&too_precise),
            Err(NumericEncoderError::ScaleExceeded(_, _))
        ));
    }
}
