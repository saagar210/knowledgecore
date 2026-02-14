use crate::app_error::{AppError, AppResult};
use crate::events::append_event;
use crate::types::{DocId, ObjectHash};
use rusqlite::{params, Connection};

#[derive(Debug, Clone)]
pub struct IngestedDoc {
    pub doc_id: DocId,
    pub original_object_hash: ObjectHash,
    pub bytes: i64,
    pub mime: String,
    pub source_kind: String,
    pub effective_ts_ms: i64,
}

pub fn ingest_bytes(
    conn: &Connection,
    object_store: &crate::object_store::ObjectStore,
    bytes: &[u8],
    mime: &str,
    source_kind: &str,
    effective_ts_ms: i64,
    source_path: Option<&str>,
    now_ms: i64,
) -> AppResult<IngestedDoc> {
    let ingest_event = append_event(
        conn,
        now_ms,
        "ingest.bytes",
        &serde_json::json!({
            "mime": mime,
            "source_kind": source_kind,
            "source_path": source_path,
            "bytes": bytes.len()
        }),
    )?;

    let original_object_hash = object_store.put_bytes(conn, bytes, ingest_event.event_id)?;
    let doc_id = DocId(original_object_hash.0.clone());

    conn.execute(
        "INSERT OR IGNORE INTO docs (doc_id, original_object_hash, bytes, mime, source_kind, effective_ts_ms, ingested_event_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            doc_id.0,
            original_object_hash.0,
            bytes.len() as i64,
            mime,
            source_kind,
            effective_ts_ms,
            ingest_event.event_id
        ],
    )
    .map_err(|e| {
        AppError::new(
            "KC_DB_INTEGRITY_FAILED",
            "ingest",
            "failed to insert doc",
            false,
            serde_json::json!({ "error": e.to_string() }),
        )
    })?;

    if let Some(path) = source_path {
        conn.execute(
            "INSERT OR IGNORE INTO doc_sources (doc_id, source_path) VALUES (?1, ?2)",
            params![doc_id.0, path],
        )
        .map_err(|e| {
            AppError::new(
                "KC_DB_INTEGRITY_FAILED",
                "ingest",
                "failed to insert doc source",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;
    }

    let row = conn
        .query_row(
            "SELECT bytes, mime, source_kind, effective_ts_ms FROM docs WHERE doc_id=?1",
            params![doc_id.0],
            |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, i64>(3)?,
                ))
            },
        )
        .map_err(|e| {
            AppError::new(
                "KC_DB_INTEGRITY_FAILED",
                "ingest",
                "failed to load ingested doc",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;

    Ok(IngestedDoc {
        doc_id,
        original_object_hash,
        bytes: row.0,
        mime: row.1,
        source_kind: row.2,
        effective_ts_ms: row.3,
    })
}
