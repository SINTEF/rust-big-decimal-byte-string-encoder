use big_decimal_byte_string_encoder::{
    decode_bigquery_bytes_to_bigdecimal, encode_bigdecimal_to_bigquery_bytes,
};
use bigdecimal::BigDecimal;
use bigdecimal::FromPrimitive;
use std::str::FromStr;

#[test]
fn test_encode_decode_integration() {
    let test_cases = vec![
        "0",
        "1.2",
        "-1.2",
        "99999999999999999999999999999.999999999",
        "-99999999999999999999999999999.999999999",
        "-123456789.42001",
        "12.345",
        "1",
        "2",
        "-1",
        "128",
        "-128",
        "12702228",
    ];

    for &value in test_cases.iter() {
        let original = BigDecimal::from_str(value).unwrap();
        let encoded = encode_bigdecimal_to_bigquery_bytes(&original).unwrap();
        let decoded = decode_bigquery_bytes_to_bigdecimal(&encoded).unwrap();
        assert_eq!(original, decoded, "Failed for value: {}", value);
    }
}

#[test]
fn test_large_numbers() {
    let large_number = "12345678901234567890.123456789";
    let original = BigDecimal::from_str(large_number).unwrap();
    let encoded = encode_bigdecimal_to_bigquery_bytes(&original).unwrap();
    let decoded = decode_bigquery_bytes_to_bigdecimal(&encoded).unwrap();
    assert_eq!(
        original, decoded,
        "Failed for large number: {}",
        large_number
    );
}

#[test]
fn test_small_numbers() {
    let small_number = "0.000000001";
    let original = BigDecimal::from_str(small_number).unwrap();
    let encoded = encode_bigdecimal_to_bigquery_bytes(&original).unwrap();
    let decoded = decode_bigquery_bytes_to_bigdecimal(&encoded).unwrap();
    assert_eq!(
        original, decoded,
        "Failed for small number: {}",
        small_number
    );
}

#[test]
fn test_overflow_error() {
    let too_big = BigDecimal::from_str("100000000000000000000000000000").unwrap();
    let result = encode_bigdecimal_to_bigquery_bytes(&too_big);
    assert!(result.is_err(), "Expected overflow error");
}

#[test]
fn test_scale_exceeded_error() {
    let too_precise = BigDecimal::from_str("1.0000000001").unwrap();
    let result = encode_bigdecimal_to_bigquery_bytes(&too_precise);
    assert!(result.is_err(), "Expected scale exceeded error");
}

#[test]
fn test_roundtrip_random() {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    for _ in 0..100 {
        let random_float: f64 = rng.gen_range(-1e29..1e29);
        let original = BigDecimal::from_f64(random_float).unwrap().with_scale(9);
        let encoded = encode_bigdecimal_to_bigquery_bytes(&original).unwrap();
        let decoded = decode_bigquery_bytes_to_bigdecimal(&encoded).unwrap();
        assert_eq!(original, decoded, "Failed for random value: {}", original);
    }
}
