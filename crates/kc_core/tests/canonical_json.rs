use kc_core::canon_json::{hash_canonical, to_canonical_bytes};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
struct VectorCase {
    name: String,
    input: Value,
    expected: String,
}

#[test]
fn canonical_json_vectors() {
    let raw = include_str!("canonical_json_vectors.json");
    let vectors: Vec<VectorCase> = serde_json::from_str(raw).expect("valid fixture json");

    for case in vectors {
        let got = to_canonical_bytes(&case.input).expect("canonical bytes");
        assert_eq!(String::from_utf8(got).expect("utf8"), case.expected, "{}", case.name);
    }
}

#[test]
fn canonical_json_float_forbidden() {
    let input = serde_json::json!({"bad": 1.5});
    let err = to_canonical_bytes(&input).expect_err("must reject floats");
    assert_eq!(err.code, "KC_CANON_JSON_FLOAT_FORBIDDEN");
}

#[test]
fn canonical_json_hash_stable() {
    let input = serde_json::json!({"b": 2, "a": [1, 2, 3]});
    let h1 = hash_canonical(&input).expect("hash");
    let h2 = hash_canonical(&input).expect("hash");
    assert_eq!(h1, h2);
    assert!(h1.starts_with("blake3:"));
}
