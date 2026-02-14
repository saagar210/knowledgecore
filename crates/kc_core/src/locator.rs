use crate::app_error::{AppError, AppResult};
use crate::types::{CanonicalHash, DocId, ObjectHash};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocatorV1 {
    pub v: i64,
    pub doc_id: DocId,
    pub canonical_hash: CanonicalHash,
    pub range: LocatorRange,
    pub hints: Option<LocatorHints>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocatorRange {
    pub start: i64,
    pub end: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocatorHints {
    pub kind: Option<String>,
    pub pages: Option<PageRange>,
    pub heading_path: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageRange {
    pub start: i64,
    pub end: i64,
}

pub fn resolve_locator_strict(
    conn: &rusqlite::Connection,
    object_store: &crate::object_store::ObjectStore,
    locator: &LocatorV1,
) -> AppResult<String> {
    if locator.v != 1 {
        return Err(AppError::new(
            "KC_LOCATOR_INVALID_SCHEMA",
            "locator",
            "unsupported locator version",
            false,
            serde_json::json!({ "expected": 1, "actual": locator.v }),
        ));
    }

    let (stored_hash, stored_obj_hash): (String, String) = conn
        .query_row(
            "SELECT canonical_hash, canonical_object_hash FROM canonical_text WHERE doc_id=?1",
            [locator.doc_id.0.clone()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|e| {
            AppError::new(
                "KC_LOCATOR_INVALID_SCHEMA",
                "locator",
                "failed to load canonical metadata for doc_id",
                false,
                serde_json::json!({ "error": e.to_string(), "doc_id": locator.doc_id.0 }),
            )
        })?;

    if stored_hash != locator.canonical_hash.0 {
        return Err(AppError::new(
            "KC_LOCATOR_CANONICAL_HASH_MISMATCH",
            "locator",
            "locator canonical hash does not match stored canonical hash",
            false,
            serde_json::json!({ "expected": stored_hash, "actual": locator.canonical_hash.0 }),
        ));
    }

    let bytes = object_store.get_bytes(&ObjectHash(stored_obj_hash))?;
    let text = String::from_utf8(bytes).map_err(|e| {
        AppError::new(
            "KC_LOCATOR_INVALID_SCHEMA",
            "locator",
            "canonical bytes are not utf8",
            false,
            serde_json::json!({ "error": e.to_string() }),
        )
    })?;

    let start = locator.range.start;
    let end = locator.range.end;
    let total = text.chars().count() as i64;
    if start < 0 || end < start || end > total {
        return Err(AppError::new(
            "KC_LOCATOR_RANGE_OOB",
            "locator",
            "locator range is outside canonical text bounds",
            false,
            serde_json::json!({ "start": start, "end": end, "len": total }),
        ));
    }

    Ok(text
        .chars()
        .skip(start as usize)
        .take((end - start) as usize)
        .collect())
}
