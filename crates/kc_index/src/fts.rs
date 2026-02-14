use kc_core::app_error::{AppError, AppResult};
use kc_core::index_traits::LexicalCandidate;
use kc_core::types::ChunkId;
use rusqlite::{params, Connection};

#[derive(Debug, Clone)]
pub struct FtsRow {
    pub chunk_id: String,
    pub doc_id: String,
    pub ordinal: i64,
    pub content: String,
}

pub fn init_fts(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "CREATE VIRTUAL TABLE IF NOT EXISTS chunks_fts
         USING fts5(chunk_id UNINDEXED, doc_id UNINDEXED, content, tokenize='unicode61');",
    )
    .map_err(|e| {
        AppError::new(
            "KC_FTS_INIT_FAILED",
            "fts",
            "failed to initialize FTS table",
            false,
            serde_json::json!({ "error": e.to_string() }),
        )
    })
}

pub fn rebuild_rows(conn: &Connection, rows: &[FtsRow]) -> AppResult<()> {
    init_fts(conn)?;
    conn.execute("DELETE FROM chunks_fts", [])
        .map_err(|e| {
            AppError::new(
                "KC_FTS_REBUILD_FAILED",
                "fts",
                "failed clearing FTS table",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;

    let mut sorted = rows.to_vec();
    sorted.sort_by(|a, b| {
        a.doc_id
            .cmp(&b.doc_id)
            .then(a.ordinal.cmp(&b.ordinal))
            .then(a.chunk_id.cmp(&b.chunk_id))
    });

    for row in sorted {
        conn.execute(
            "INSERT INTO chunks_fts(chunk_id, doc_id, content) VALUES (?1, ?2, ?3)",
            params![row.chunk_id, row.doc_id, row.content],
        )
        .map_err(|e| {
            AppError::new(
                "KC_FTS_REBUILD_FAILED",
                "fts",
                "failed inserting FTS row",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;
    }
    Ok(())
}

pub fn query(conn: &Connection, q: &str, limit: usize) -> AppResult<Vec<LexicalCandidate>> {
    let mut stmt = conn
        .prepare("SELECT chunk_id, rank FROM chunks_fts WHERE chunks_fts MATCH ?1 ORDER BY rank LIMIT ?2")
        .map_err(|e| {
            AppError::new(
                "KC_FTS_QUERY_FAILED",
                "fts",
                "failed to prepare FTS query",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;

    let rows = stmt
        .query_map(params![q, limit as i64], |row| {
            let chunk_id: String = row.get(0)?;
            let rank: f64 = row.get(1)?;
            Ok((chunk_id, rank))
        })
        .map_err(|e| {
            AppError::new(
                "KC_FTS_QUERY_FAILED",
                "fts",
                "failed running FTS query",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;

    let mut out = Vec::new();
    for (idx, row) in rows.enumerate() {
        let (chunk_id, _rank) = row.map_err(|e| {
            AppError::new(
                "KC_FTS_QUERY_FAILED",
                "fts",
                "failed reading FTS row",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;
        out.push(LexicalCandidate {
            chunk_id: ChunkId(chunk_id),
            rank: idx as i64 + 1,
        });
    }

    Ok(out)
}
