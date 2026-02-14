use crate::app_error::{AppError, AppResult};
use crate::hashing::blake3_hex_prefixed;
use crate::lineage_governance::ensure_lineage_permission;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LineageNodeV1 {
    pub node_id: String,
    pub kind: String,
    pub label: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LineageEdgeV1 {
    pub from_node_id: String,
    pub to_node_id: String,
    pub relation: String,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LineageQueryResV1 {
    pub schema_version: i64,
    pub seed_doc_id: String,
    pub depth: i64,
    pub generated_at_ms: i64,
    pub nodes: Vec<LineageNodeV1>,
    pub edges: Vec<LineageEdgeV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LineageOverlayEntryV1 {
    pub overlay_id: String,
    pub doc_id: String,
    pub from_node_id: String,
    pub to_node_id: String,
    pub relation: String,
    pub evidence: String,
    pub created_at_ms: i64,
    pub created_by: String,
}

pub const LINEAGE_LOCK_LEASE_MS: i64 = 15 * 60 * 1000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LineageLockLeaseV1 {
    pub doc_id: String,
    pub owner: String,
    pub token: String,
    pub acquired_at_ms: i64,
    pub expires_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LineageLockStatusV1 {
    pub doc_id: String,
    pub held: bool,
    pub owner: Option<String>,
    pub acquired_at_ms: Option<i64>,
    pub expires_at_ms: Option<i64>,
    pub expired: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LineageEdgeV2 {
    pub from_node_id: String,
    pub to_node_id: String,
    pub relation: String,
    pub evidence: String,
    pub origin: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LineageQueryResV2 {
    pub schema_version: i64,
    pub seed_doc_id: String,
    pub depth: i64,
    pub generated_at_ms: i64,
    pub nodes: Vec<LineageNodeV1>,
    pub edges: Vec<LineageEdgeV2>,
}

#[derive(Debug, Clone)]
struct EventRow {
    event_id: i64,
    ts_ms: i64,
    event_type: String,
    payload_json: String,
    prev_event_hash: Option<String>,
    event_hash: String,
}

fn lineage_error(code: &str, message: &str, details: serde_json::Value) -> AppError {
    AppError::new(code, "lineage", message, false, details)
}

fn add_node(
    nodes_by_id: &mut BTreeMap<String, LineageNodeV1>,
    node_id: String,
    kind: &str,
    label: String,
    metadata: serde_json::Value,
) {
    nodes_by_id.entry(node_id.clone()).or_insert(LineageNodeV1 {
        node_id,
        kind: kind.to_string(),
        label,
        metadata,
    });
}

fn add_edge(
    edge_keys: &mut BTreeSet<(String, String, String, String)>,
    from_node_id: String,
    to_node_id: String,
    relation: &str,
    evidence: String,
) {
    edge_keys.insert((from_node_id, to_node_id, relation.to_string(), evidence));
}

fn load_event_by_id(conn: &Connection, event_id: i64) -> AppResult<Option<EventRow>> {
    let out = conn.query_row(
        "SELECT event_id, ts_ms, type, payload_json, prev_event_hash, event_hash
         FROM events WHERE event_id=?1",
        params![event_id],
        |row| {
            Ok(EventRow {
                event_id: row.get(0)?,
                ts_ms: row.get(1)?,
                event_type: row.get(2)?,
                payload_json: row.get(3)?,
                prev_event_hash: row.get(4)?,
                event_hash: row.get(5)?,
            })
        },
    );

    match out {
        Ok(row) => Ok(Some(row)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(lineage_error(
            "KC_LINEAGE_QUERY_FAILED",
            "failed loading event by id",
            serde_json::json!({ "error": e.to_string(), "event_id": event_id }),
        )),
    }
}

fn load_event_by_hash(conn: &Connection, event_hash: &str) -> AppResult<Option<EventRow>> {
    let out = conn.query_row(
        "SELECT event_id, ts_ms, type, payload_json, prev_event_hash, event_hash
         FROM events WHERE event_hash=?1",
        params![event_hash],
        |row| {
            Ok(EventRow {
                event_id: row.get(0)?,
                ts_ms: row.get(1)?,
                event_type: row.get(2)?,
                payload_json: row.get(3)?,
                prev_event_hash: row.get(4)?,
                event_hash: row.get(5)?,
            })
        },
    );

    match out {
        Ok(row) => Ok(Some(row)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(lineage_error(
            "KC_LINEAGE_QUERY_FAILED",
            "failed loading event by hash",
            serde_json::json!({ "error": e.to_string(), "event_hash": event_hash }),
        )),
    }
}

fn insert_event_node(
    nodes_by_id: &mut BTreeMap<String, LineageNodeV1>,
    event: &EventRow,
) -> String {
    let node_id = format!("event:{}", event.event_id);
    add_node(
        nodes_by_id,
        node_id.clone(),
        "event",
        format!("Event {} {}", event.event_id, event.event_type),
        serde_json::json!({
            "event_id": event.event_id,
            "ts_ms": event.ts_ms,
            "event_type": event.event_type,
            "payload_json": event.payload_json,
            "event_hash": event.event_hash,
            "prev_event_hash": event.prev_event_hash,
        }),
    );
    node_id
}

fn append_event_chain(
    conn: &Connection,
    start_event_id: i64,
    depth: i64,
    nodes_by_id: &mut BTreeMap<String, LineageNodeV1>,
    edge_keys: &mut BTreeSet<(String, String, String, String)>,
) -> AppResult<()> {
    if depth <= 1 {
        return Ok(());
    }

    let mut steps = 1i64;
    let mut current = match load_event_by_id(conn, start_event_id)? {
        Some(event) => event,
        None => return Ok(()),
    };

    let mut current_id = insert_event_node(nodes_by_id, &current);

    while steps < depth {
        let Some(prev_hash) = current.prev_event_hash.clone() else {
            break;
        };
        let Some(prev_event) = load_event_by_hash(conn, &prev_hash)? else {
            break;
        };

        let prev_id = insert_event_node(nodes_by_id, &prev_event);
        add_edge(
            edge_keys,
            current_id.clone(),
            prev_id.clone(),
            "prev_event",
            prev_hash,
        );

        current = prev_event;
        current_id = prev_id;
        steps += 1;
    }

    Ok(())
}

#[derive(Debug, Clone)]
struct LockRow {
    owner: String,
    token: String,
    acquired_at_ms: i64,
    expires_at_ms: i64,
}

fn lock_token_for(doc_id: &str, owner: &str, now_ms: i64) -> String {
    blake3_hex_prefixed(format!("kc.lineage.lock.v1\n{doc_id}\n{owner}\n{now_ms}").as_bytes())
}

fn read_lock_row(conn: &Connection, doc_id: &str) -> AppResult<Option<LockRow>> {
    let out = conn.query_row(
        "SELECT owner, token, acquired_at_ms, expires_at_ms
         FROM lineage_edit_locks
         WHERE doc_id=?1",
        params![doc_id],
        |row| {
            Ok(LockRow {
                owner: row.get(0)?,
                token: row.get(1)?,
                acquired_at_ms: row.get(2)?,
                expires_at_ms: row.get(3)?,
            })
        },
    );
    match out {
        Ok(row) => Ok(Some(row)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(lineage_error(
            "KC_LINEAGE_QUERY_FAILED",
            "failed reading lineage edit lock",
            serde_json::json!({ "error": e.to_string(), "doc_id": doc_id }),
        )),
    }
}

pub fn query_lineage(
    conn: &Connection,
    seed_doc_id: &str,
    depth: i64,
    now_ms: i64,
) -> AppResult<LineageQueryResV1> {
    if depth <= 0 {
        return Err(lineage_error(
            "KC_LINEAGE_INVALID_DEPTH",
            "lineage depth must be >= 1",
            serde_json::json!({ "depth": depth, "min": 1 }),
        ));
    }

    let doc_row = conn.query_row(
        "SELECT original_object_hash, ingested_event_id, mime, source_kind, effective_ts_ms
         FROM docs WHERE doc_id=?1",
        params![seed_doc_id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
            ))
        },
    );

    let (original_object_hash, ingested_event_id, mime, source_kind, effective_ts_ms) =
        match doc_row {
            Ok(row) => row,
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                return Err(lineage_error(
                    "KC_LINEAGE_DOC_NOT_FOUND",
                    "seed document does not exist",
                    serde_json::json!({ "seed_doc_id": seed_doc_id }),
                ))
            }
            Err(e) => {
                return Err(lineage_error(
                    "KC_LINEAGE_QUERY_FAILED",
                    "failed loading seed doc",
                    serde_json::json!({ "error": e.to_string(), "seed_doc_id": seed_doc_id }),
                ))
            }
        };

    let mut nodes_by_id: BTreeMap<String, LineageNodeV1> = BTreeMap::new();
    let mut edge_keys: BTreeSet<(String, String, String, String)> = BTreeSet::new();

    let seed_node_id = format!("doc:{seed_doc_id}");
    add_node(
        &mut nodes_by_id,
        seed_node_id.clone(),
        "doc",
        seed_doc_id.to_string(),
        serde_json::json!({
            "doc_id": seed_doc_id,
            "mime": mime,
            "source_kind": source_kind,
            "effective_ts_ms": effective_ts_ms
        }),
    );

    let original_object_node = format!("object:{original_object_hash}");
    add_node(
        &mut nodes_by_id,
        original_object_node.clone(),
        "object",
        original_object_hash.clone(),
        serde_json::json!({
            "object_hash": original_object_hash,
            "role": "original"
        }),
    );
    add_edge(
        &mut edge_keys,
        seed_node_id.clone(),
        original_object_node,
        "originates_from",
        "docs.original_object_hash".to_string(),
    );

    let mut source_stmt = conn
        .prepare("SELECT source_path FROM doc_sources WHERE doc_id=?1 ORDER BY source_path ASC")
        .map_err(|e| {
            lineage_error(
                "KC_LINEAGE_QUERY_FAILED",
                "failed preparing source query",
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;
    let source_rows = source_stmt
        .query_map(params![seed_doc_id], |row| row.get::<_, String>(0))
        .map_err(|e| {
            lineage_error(
                "KC_LINEAGE_QUERY_FAILED",
                "failed querying sources",
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;
    for row in source_rows {
        let source_path = row.map_err(|e| {
            lineage_error(
                "KC_LINEAGE_QUERY_FAILED",
                "failed decoding source row",
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;
        let source_node_id = format!("source:{source_path}");
        add_node(
            &mut nodes_by_id,
            source_node_id.clone(),
            "source",
            source_path.clone(),
            serde_json::json!({ "source_path": source_path }),
        );
        add_edge(
            &mut edge_keys,
            seed_node_id.clone(),
            source_node_id,
            "source_path",
            "doc_sources".to_string(),
        );
    }

    let canonical_row = conn.query_row(
        "SELECT canonical_hash, canonical_object_hash, extractor_name, extractor_version, normalization_version, toolchain_json, created_event_id
         FROM canonical_text WHERE doc_id=?1",
        params![seed_doc_id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, i64>(6)?,
            ))
        },
    );
    match canonical_row {
        Ok((
            canonical_hash,
            canonical_object_hash,
            extractor_name,
            extractor_version,
            normalization_version,
            toolchain_json,
            created_event_id,
        )) => {
            let canonical_node = format!("canonical:{canonical_hash}");
            add_node(
                &mut nodes_by_id,
                canonical_node.clone(),
                "canonical",
                canonical_hash.clone(),
                serde_json::json!({
                    "canonical_hash": canonical_hash,
                    "extractor_name": extractor_name,
                    "extractor_version": extractor_version,
                    "normalization_version": normalization_version,
                    "toolchain_json": toolchain_json,
                }),
            );
            add_edge(
                &mut edge_keys,
                seed_node_id.clone(),
                canonical_node.clone(),
                "canonical_text",
                "canonical_text".to_string(),
            );

            let canonical_obj_node = format!("object:{canonical_object_hash}");
            add_node(
                &mut nodes_by_id,
                canonical_obj_node.clone(),
                "object",
                canonical_object_hash,
                serde_json::json!({
                    "role": "canonical"
                }),
            );
            add_edge(
                &mut edge_keys,
                canonical_node.clone(),
                canonical_obj_node,
                "stored_as",
                "canonical_text.canonical_object_hash".to_string(),
            );

            if let Some(canonical_event) = load_event_by_id(conn, created_event_id)? {
                let event_node = insert_event_node(&mut nodes_by_id, &canonical_event);
                add_edge(
                    &mut edge_keys,
                    canonical_node,
                    event_node,
                    "created_by_event",
                    "canonical_text.created_event_id".to_string(),
                );
                append_event_chain(
                    conn,
                    created_event_id,
                    depth,
                    &mut nodes_by_id,
                    &mut edge_keys,
                )?;
            }
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {}
        Err(e) => {
            return Err(lineage_error(
                "KC_LINEAGE_QUERY_FAILED",
                "failed loading canonical lineage",
                serde_json::json!({ "error": e.to_string() }),
            ))
        }
    }

    let mut chunk_stmt = conn
        .prepare(
            "SELECT chunk_id, ordinal, start_char, end_char, chunking_config_hash
             FROM chunks WHERE doc_id=?1 ORDER BY ordinal ASC, chunk_id ASC",
        )
        .map_err(|e| {
            lineage_error(
                "KC_LINEAGE_QUERY_FAILED",
                "failed preparing chunk query",
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;
    let chunk_rows = chunk_stmt
        .query_map(params![seed_doc_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, String>(4)?,
            ))
        })
        .map_err(|e| {
            lineage_error(
                "KC_LINEAGE_QUERY_FAILED",
                "failed querying chunks",
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;
    for row in chunk_rows {
        let (chunk_id, ordinal, start_char, end_char, chunking_config_hash) = row.map_err(|e| {
            lineage_error(
                "KC_LINEAGE_QUERY_FAILED",
                "failed decoding chunk row",
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;
        let chunk_node = format!("chunk:{chunk_id}");
        add_node(
            &mut nodes_by_id,
            chunk_node.clone(),
            "chunk",
            format!("Chunk {ordinal}"),
            serde_json::json!({
                "chunk_id": chunk_id,
                "ordinal": ordinal,
                "start_char": start_char,
                "end_char": end_char,
                "chunking_config_hash": chunking_config_hash
            }),
        );
        add_edge(
            &mut edge_keys,
            seed_node_id.clone(),
            chunk_node,
            "contains_chunk",
            format!("ordinal:{ordinal}"),
        );
    }

    if let Some(ingest_event) = load_event_by_id(conn, ingested_event_id)? {
        let ingest_event_node = insert_event_node(&mut nodes_by_id, &ingest_event);
        add_edge(
            &mut edge_keys,
            seed_node_id.clone(),
            ingest_event_node,
            "ingested_by_event",
            "docs.ingested_event_id".to_string(),
        );
        append_event_chain(
            conn,
            ingested_event_id,
            depth,
            &mut nodes_by_id,
            &mut edge_keys,
        )?;
    }

    let mut nodes: Vec<LineageNodeV1> = nodes_by_id.into_values().collect();
    nodes.sort_by(|a, b| a.kind.cmp(&b.kind).then(a.node_id.cmp(&b.node_id)));

    let edges: Vec<LineageEdgeV1> = edge_keys
        .into_iter()
        .map(
            |(from_node_id, to_node_id, relation, evidence)| LineageEdgeV1 {
                from_node_id,
                to_node_id,
                relation,
                evidence,
            },
        )
        .collect();

    Ok(LineageQueryResV1 {
        schema_version: 1,
        seed_doc_id: seed_doc_id.to_string(),
        depth,
        generated_at_ms: now_ms,
        nodes,
        edges,
    })
}

pub fn lineage_lock_acquire(
    conn: &Connection,
    doc_id: &str,
    owner: &str,
    now_ms: i64,
) -> AppResult<LineageLockLeaseV1> {
    if doc_id.trim().is_empty() || owner.trim().is_empty() {
        return Err(lineage_error(
            "KC_LINEAGE_LOCK_INVALID",
            "lineage lock acquire requires non-empty doc_id and owner",
            serde_json::json!({ "doc_id": doc_id, "owner": owner }),
        ));
    }

    if let Some(lock) = read_lock_row(conn, doc_id)? {
        if lock.expires_at_ms > now_ms {
            return Err(lineage_error(
                "KC_LINEAGE_LOCK_HELD",
                "lineage edit lock is already held",
                serde_json::json!({
                    "doc_id": doc_id,
                    "owner": lock.owner,
                    "expires_at_ms": lock.expires_at_ms
                }),
            ));
        }
    }

    let token = lock_token_for(doc_id, owner, now_ms);
    let expires_at_ms = now_ms + LINEAGE_LOCK_LEASE_MS;
    conn.execute(
        "INSERT INTO lineage_edit_locks(doc_id, owner, token, acquired_at_ms, expires_at_ms)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(doc_id) DO UPDATE SET
           owner=excluded.owner,
           token=excluded.token,
           acquired_at_ms=excluded.acquired_at_ms,
           expires_at_ms=excluded.expires_at_ms",
        params![doc_id, owner, token, now_ms, expires_at_ms],
    )
    .map_err(|e| {
        lineage_error(
            "KC_LINEAGE_QUERY_FAILED",
            "failed writing lineage edit lock",
            serde_json::json!({ "error": e.to_string(), "doc_id": doc_id }),
        )
    })?;

    Ok(LineageLockLeaseV1 {
        doc_id: doc_id.to_string(),
        owner: owner.to_string(),
        token,
        acquired_at_ms: now_ms,
        expires_at_ms,
    })
}

pub fn lineage_lock_release(conn: &Connection, doc_id: &str, token: &str) -> AppResult<()> {
    let Some(lock) = read_lock_row(conn, doc_id)? else {
        return Err(lineage_error(
            "KC_LINEAGE_LOCK_INVALID",
            "lineage edit lock does not exist",
            serde_json::json!({ "doc_id": doc_id }),
        ));
    };
    if lock.token != token {
        return Err(lineage_error(
            "KC_LINEAGE_LOCK_INVALID",
            "lineage edit lock token is invalid",
            serde_json::json!({ "doc_id": doc_id }),
        ));
    }
    conn.execute(
        "DELETE FROM lineage_edit_locks WHERE doc_id=?1",
        params![doc_id],
    )
    .map_err(|e| {
        lineage_error(
            "KC_LINEAGE_QUERY_FAILED",
            "failed deleting lineage edit lock",
            serde_json::json!({ "error": e.to_string(), "doc_id": doc_id }),
        )
    })?;
    Ok(())
}

pub fn lineage_lock_status(
    conn: &Connection,
    doc_id: &str,
    now_ms: i64,
) -> AppResult<LineageLockStatusV1> {
    let Some(lock) = read_lock_row(conn, doc_id)? else {
        return Ok(LineageLockStatusV1 {
            doc_id: doc_id.to_string(),
            held: false,
            owner: None,
            acquired_at_ms: None,
            expires_at_ms: None,
            expired: false,
        });
    };
    let expired = lock.expires_at_ms <= now_ms;
    Ok(LineageLockStatusV1 {
        doc_id: doc_id.to_string(),
        held: !expired,
        owner: Some(lock.owner),
        acquired_at_ms: Some(lock.acquired_at_ms),
        expires_at_ms: Some(lock.expires_at_ms),
        expired,
    })
}

fn require_valid_lock(
    conn: &Connection,
    doc_id: &str,
    token: &str,
    now_ms: i64,
) -> AppResult<LockRow> {
    let status = lineage_lock_status(conn, doc_id, now_ms)?;
    if !status.held {
        return Err(lineage_error(
            if status.expired {
                "KC_LINEAGE_LOCK_EXPIRED"
            } else {
                "KC_LINEAGE_LOCK_INVALID"
            },
            "lineage edit lock is required for overlay mutation",
            serde_json::json!({ "doc_id": doc_id }),
        ));
    }

    let Some(lock) = read_lock_row(conn, doc_id)? else {
        return Err(lineage_error(
            "KC_LINEAGE_LOCK_INVALID",
            "lineage edit lock is required for overlay mutation",
            serde_json::json!({ "doc_id": doc_id }),
        ));
    };

    if lock.token != token {
        return Err(lineage_error(
            "KC_LINEAGE_LOCK_INVALID",
            "lineage edit lock token is invalid",
            serde_json::json!({ "doc_id": doc_id }),
        ));
    }

    Ok(lock)
}

fn overlay_id_for(
    doc_id: &str,
    from_node_id: &str,
    to_node_id: &str,
    relation: &str,
    evidence: &str,
) -> String {
    blake3_hex_prefixed(
        format!(
            "kc.lineage.overlay.v1\n{}\n{}\n{}\n{}\n{}",
            doc_id, from_node_id, to_node_id, relation, evidence
        )
        .as_bytes(),
    )
}

fn validate_overlay_fields(
    doc_id: &str,
    from_node_id: &str,
    to_node_id: &str,
    relation: &str,
    evidence: &str,
    created_by: &str,
) -> AppResult<()> {
    let fields = [
        ("doc_id", doc_id),
        ("from_node_id", from_node_id),
        ("to_node_id", to_node_id),
        ("relation", relation),
        ("evidence", evidence),
        ("created_by", created_by),
    ];
    if let Some((name, _)) = fields.iter().find(|(_, v)| v.trim().is_empty()) {
        return Err(lineage_error(
            "KC_LINEAGE_OVERLAY_INVALID",
            "overlay field must not be empty",
            serde_json::json!({ "field": name }),
        ));
    }
    Ok(())
}

pub fn lineage_overlay_add(
    conn: &Connection,
    doc_id: &str,
    from_node_id: &str,
    to_node_id: &str,
    relation: &str,
    evidence: &str,
    lock_token: &str,
    created_at_ms: i64,
    created_by: &str,
) -> AppResult<LineageOverlayEntryV1> {
    validate_overlay_fields(
        doc_id,
        from_node_id,
        to_node_id,
        relation,
        evidence,
        created_by,
    )?;
    let _lock = require_valid_lock(conn, doc_id, lock_token, created_at_ms)?;
    ensure_lineage_permission(conn, created_by, "lineage.overlay.write", Some(doc_id))?;
    let overlay_id = overlay_id_for(doc_id, from_node_id, to_node_id, relation, evidence);
    let inserted = conn.execute(
        "INSERT INTO lineage_overlays(
            overlay_id, doc_id, from_node_id, to_node_id, relation, evidence, created_at_ms, created_by
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            overlay_id,
            doc_id,
            from_node_id,
            to_node_id,
            relation,
            evidence,
            created_at_ms,
            created_by
        ],
    );
    match inserted {
        Ok(_) => Ok(LineageOverlayEntryV1 {
            overlay_id: overlay_id_for(doc_id, from_node_id, to_node_id, relation, evidence),
            doc_id: doc_id.to_string(),
            from_node_id: from_node_id.to_string(),
            to_node_id: to_node_id.to_string(),
            relation: relation.to_string(),
            evidence: evidence.to_string(),
            created_at_ms,
            created_by: created_by.to_string(),
        }),
        Err(rusqlite::Error::SqliteFailure(err, _))
            if err.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE =>
        {
            Err(lineage_error(
                "KC_LINEAGE_OVERLAY_CONFLICT",
                "overlay already exists",
                serde_json::json!({
                    "overlay_id": overlay_id_for(doc_id, from_node_id, to_node_id, relation, evidence),
                    "doc_id": doc_id,
                    "from_node_id": from_node_id,
                    "to_node_id": to_node_id,
                    "relation": relation,
                    "evidence": evidence
                }),
            ))
        }
        Err(e) => Err(lineage_error(
            "KC_LINEAGE_QUERY_FAILED",
            "failed inserting lineage overlay",
            serde_json::json!({ "error": e.to_string() }),
        )),
    }
}

pub fn lineage_overlay_remove(
    conn: &Connection,
    overlay_id: &str,
    lock_token: &str,
    now_ms: i64,
) -> AppResult<()> {
    let doc_id = match conn.query_row(
        "SELECT doc_id FROM lineage_overlays WHERE overlay_id=?1",
        params![overlay_id],
        |row| row.get::<_, String>(0),
    ) {
        Ok(doc_id) => doc_id,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            return Err(lineage_error(
                "KC_LINEAGE_OVERLAY_NOT_FOUND",
                "lineage overlay not found",
                serde_json::json!({ "overlay_id": overlay_id }),
            ));
        }
        Err(e) => {
            return Err(lineage_error(
                "KC_LINEAGE_QUERY_FAILED",
                "failed loading lineage overlay before delete",
                serde_json::json!({ "error": e.to_string(), "overlay_id": overlay_id }),
            ));
        }
    };
    let lock = require_valid_lock(conn, &doc_id, lock_token, now_ms)?;
    ensure_lineage_permission(conn, &lock.owner, "lineage.overlay.write", Some(&doc_id))?;

    let removed = conn
        .execute(
            "DELETE FROM lineage_overlays WHERE overlay_id=?1",
            params![overlay_id],
        )
        .map_err(|e| {
            lineage_error(
                "KC_LINEAGE_QUERY_FAILED",
                "failed deleting lineage overlay",
                serde_json::json!({ "error": e.to_string(), "overlay_id": overlay_id }),
            )
        })?;
    if removed == 0 {
        return Err(lineage_error(
            "KC_LINEAGE_OVERLAY_NOT_FOUND",
            "lineage overlay not found",
            serde_json::json!({ "overlay_id": overlay_id }),
        ));
    }
    Ok(())
}

pub fn lineage_overlay_list(
    conn: &Connection,
    doc_id: &str,
) -> AppResult<Vec<LineageOverlayEntryV1>> {
    let mut stmt = conn
        .prepare(
            "SELECT overlay_id, doc_id, from_node_id, to_node_id, relation, evidence, created_at_ms, created_by
             FROM lineage_overlays
             WHERE doc_id=?1
             ORDER BY from_node_id ASC, to_node_id ASC, relation ASC, evidence ASC, overlay_id ASC",
        )
        .map_err(|e| {
            lineage_error(
                "KC_LINEAGE_QUERY_FAILED",
                "failed preparing lineage overlay list query",
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;
    let rows = stmt
        .query_map(params![doc_id], |row| {
            Ok(LineageOverlayEntryV1 {
                overlay_id: row.get(0)?,
                doc_id: row.get(1)?,
                from_node_id: row.get(2)?,
                to_node_id: row.get(3)?,
                relation: row.get(4)?,
                evidence: row.get(5)?,
                created_at_ms: row.get(6)?,
                created_by: row.get(7)?,
            })
        })
        .map_err(|e| {
            lineage_error(
                "KC_LINEAGE_QUERY_FAILED",
                "failed querying lineage overlays",
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;
    let mut overlays = Vec::new();
    for row in rows {
        overlays.push(row.map_err(|e| {
            lineage_error(
                "KC_LINEAGE_QUERY_FAILED",
                "failed decoding lineage overlay row",
                serde_json::json!({ "error": e.to_string() }),
            )
        })?);
    }
    Ok(overlays)
}

fn inferred_node_kind(node_id: &str) -> String {
    node_id
        .split_once(':')
        .map(|(prefix, _)| prefix.to_string())
        .unwrap_or_else(|| "overlay".to_string())
}

pub fn query_lineage_v2(
    conn: &Connection,
    seed_doc_id: &str,
    depth: i64,
    now_ms: i64,
) -> AppResult<LineageQueryResV2> {
    let base = query_lineage(conn, seed_doc_id, depth, now_ms)?;
    let overlays = lineage_overlay_list(conn, seed_doc_id)?;

    let mut nodes_by_id: BTreeMap<String, LineageNodeV1> = base
        .nodes
        .into_iter()
        .map(|n| (n.node_id.clone(), n))
        .collect();
    let mut edges: Vec<LineageEdgeV2> = base
        .edges
        .into_iter()
        .map(|e| LineageEdgeV2 {
            from_node_id: e.from_node_id,
            to_node_id: e.to_node_id,
            relation: e.relation,
            evidence: e.evidence,
            origin: "system".to_string(),
        })
        .collect();

    for overlay in overlays {
        if !nodes_by_id.contains_key(&overlay.from_node_id) {
            let kind = inferred_node_kind(&overlay.from_node_id);
            nodes_by_id.insert(
                overlay.from_node_id.clone(),
                LineageNodeV1 {
                    node_id: overlay.from_node_id.clone(),
                    kind,
                    label: overlay.from_node_id.clone(),
                    metadata: serde_json::json!({ "overlay_only": true }),
                },
            );
        }
        if !nodes_by_id.contains_key(&overlay.to_node_id) {
            let kind = inferred_node_kind(&overlay.to_node_id);
            nodes_by_id.insert(
                overlay.to_node_id.clone(),
                LineageNodeV1 {
                    node_id: overlay.to_node_id.clone(),
                    kind,
                    label: overlay.to_node_id.clone(),
                    metadata: serde_json::json!({ "overlay_only": true }),
                },
            );
        }
        edges.push(LineageEdgeV2 {
            from_node_id: overlay.from_node_id,
            to_node_id: overlay.to_node_id,
            relation: overlay.relation,
            evidence: overlay.evidence,
            origin: "overlay".to_string(),
        });
    }

    let mut nodes: Vec<LineageNodeV1> = nodes_by_id.into_values().collect();
    nodes.sort_by(|a, b| a.kind.cmp(&b.kind).then(a.node_id.cmp(&b.node_id)));
    edges.sort_by(|a, b| {
        a.from_node_id
            .cmp(&b.from_node_id)
            .then(a.to_node_id.cmp(&b.to_node_id))
            .then(a.relation.cmp(&b.relation))
            .then(a.evidence.cmp(&b.evidence))
            .then(a.origin.cmp(&b.origin))
    });

    Ok(LineageQueryResV2 {
        schema_version: 2,
        seed_doc_id: seed_doc_id.to_string(),
        depth,
        generated_at_ms: now_ms,
        nodes,
        edges,
    })
}
