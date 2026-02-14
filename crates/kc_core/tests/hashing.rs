use kc_core::hashing::{blake3_hex_prefixed, validate_blake3_prefixed};

#[test]
fn hashing_blake3_prefixed_format() {
    let digest = blake3_hex_prefixed(b"hello world");
    assert_eq!(digest.len(), 71);
    assert!(digest.starts_with("blake3:"));
    validate_blake3_prefixed(&digest).expect("valid digest");
}

#[test]
fn hashing_invalid_prefix_rejected() {
    let err = validate_blake3_prefixed("sha256:abc").expect_err("invalid prefix");
    assert_eq!(err.code, "KC_HASH_INVALID_FORMAT");
}

#[test]
fn hashing_invalid_hex_rejected() {
    let err = validate_blake3_prefixed("blake3:ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789abcdef").expect_err("invalid hex");
    assert_eq!(err.code, "KC_HASH_DECODE_FAILED");
}
