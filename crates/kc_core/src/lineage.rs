use crate::app_error::{AppError, AppResult};
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
    edge_keys.insert((
        from_node_id,
        to_node_id,
        relation.to_string(),
        evidence,
    ));
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
