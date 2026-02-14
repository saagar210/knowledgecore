use crate::app_error::{AppError, AppResult};
use serde_json::Value;

fn write_canonical(value: &Value, out: &mut String) -> AppResult<()> {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => {
            if n.is_f64() {
                return Err(AppError::new(
                    "KC_CANON_JSON_FLOAT_FORBIDDEN",
                    "canon_json",
                    "floats are forbidden in canonical json",
                    false,
                    serde_json::json!({}),
                ));
            }
            out.push_str(&n.to_string());
        }
        Value::String(s) => {
            out.push_str(&serde_json::to_string(s).map_err(|_| {
                AppError::new(
                    "KC_CANON_JSON_PARSE_FAILED",
                    "canon_json",
                    "failed to encode string",
                    false,
                    serde_json::json!({}),
                )
            })?);
        }
        Value::Array(items) => {
            out.push('[');
            for (idx, item) in items.iter().enumerate() {
                if idx > 0 {
                    out.push(',');
                }
                write_canonical(item, out)?;
            }
            out.push(']');
        }
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            out.push('{');
            for (idx, key) in keys.iter().enumerate() {
                if idx > 0 {
                    out.push(',');
                }
                out.push_str(&serde_json::to_string(key).map_err(|_| {
                    AppError::new(
                        "KC_CANON_JSON_PARSE_FAILED",
                        "canon_json",
                        "failed to encode key",
                        false,
                        serde_json::json!({}),
                    )
                })?);
                out.push(':');
                write_canonical(&map[*key], out)?;
            }
            out.push('}');
        }
    }
    Ok(())
}

pub fn to_canonical_bytes(value: &Value) -> AppResult<Vec<u8>> {
    let mut out = String::new();
    write_canonical(value, &mut out)?;
    Ok(out.into_bytes())
}

pub fn hash_canonical(value: &Value) -> AppResult<String> {
    let bytes = to_canonical_bytes(value)?;
    Ok(format!("blake3:{}", blake3::hash(&bytes).to_hex()))
}
