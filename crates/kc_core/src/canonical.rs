use crate::app_error::{AppError, AppResult};
use crate::hashing::blake3_hex_prefixed;
use crate::services::CanonicalTextArtifact;
use crate::types::DocId;
use rusqlite::{params, Connection};

pub fn persist_canonical_text(
    conn: &Connection,
    object_store: &crate::object_store::ObjectStore,
    artifact: &CanonicalTextArtifact,
    created_event_id: i64,
) -> AppResult<()> {
    let computed = blake3_hex_prefixed(&artifact.canonical_bytes);
    if computed != artifact.canonical_hash.0 || computed != artifact.canonical_object_hash.0 {
        return Err(AppError::new(
            "KC_DB_INTEGRITY_FAILED",
            "canonical",
            "canonical hash invariant failed",
            false,
            serde_json::json!({
                "computed": computed,
                "canonical_hash": artifact.canonical_hash.0,
                "canonical_object_hash": artifact.canonical_object_hash.0
            }),
        ));
    }

    let stored_hash = object_store.put_bytes(conn, &artifact.canonical_bytes, created_event_id)?;

    conn.execute(
        "INSERT INTO canonical_text (
          doc_id,
          canonical_object_hash,
          canonical_hash,
          extractor_name,
          extractor_version,
          extractor_flags_json,
          normalization_version,
          toolchain_json,
          created_event_id
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        ON CONFLICT(doc_id) DO UPDATE SET
          canonical_object_hash=excluded.canonical_object_hash,
          canonical_hash=excluded.canonical_hash,
          extractor_name=excluded.extractor_name,
          extractor_version=excluded.extractor_version,
          extractor_flags_json=excluded.extractor_flags_json,
          normalization_version=excluded.normalization_version,
          toolchain_json=excluded.toolchain_json,
          created_event_id=excluded.created_event_id",
        params![
            artifact.doc_id.0,
            stored_hash.0,
            artifact.canonical_hash.0,
            artifact.extractor_name,
            artifact.extractor_version,
            artifact.extractor_flags_json,
            artifact.normalization_version,
            artifact.toolchain_json,
            created_event_id
        ],
    )
    .map_err(|e| {
        AppError::new(
            "KC_DB_INTEGRITY_FAILED",
            "canonical",
            "failed to upsert canonical_text row",
            false,
            serde_json::json!({ "error": e.to_string() }),
        )
    })?;

    Ok(())
}

pub fn load_canonical_text(
    conn: &Connection,
    object_store: &crate::object_store::ObjectStore,
    doc_id: &DocId,
) -> AppResult<Vec<u8>> {
    let hash: String = conn
        .query_row(
            "SELECT canonical_object_hash FROM canonical_text WHERE doc_id=?1",
            [doc_id.0.clone()],
            |row| row.get(0),
        )
        .map_err(|e| {
            AppError::new(
                "KC_DB_INTEGRITY_FAILED",
                "canonical",
                "failed to load canonical_object_hash",
                false,
                serde_json::json!({ "error": e.to_string(), "doc_id": doc_id.0 }),
            )
        })?;

    object_store.get_bytes(&crate::types::ObjectHash(hash))
}
