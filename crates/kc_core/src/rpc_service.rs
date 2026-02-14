use crate::app_error::{AppError, AppResult};
use crate::canonical::load_canonical_text;
use crate::db::open_db;
use crate::hashing::blake3_hex_prefixed;
use crate::ingest::ingest_bytes;
use crate::locator::{resolve_locator_strict, LocatorV1};
use crate::object_store::ObjectStore;
use crate::types::DocId;
use crate::vault::{vault_init, vault_open, vault_paths};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

static ACTIVE_JOBS: OnceLock<Mutex<BTreeSet<String>>> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct VaultSummary {
    pub vault_id: String,
    pub vault_slug: String,
}

#[derive(Debug, Clone)]
pub struct SearchHit {
    pub doc_id: String,
    pub score: f64,
    pub snippet: String,
}

#[derive(Debug, Clone)]
pub struct EventItem {
    pub event_id: i64,
    pub ts_ms: i64,
    pub event_type: String,
}

#[derive(Debug, Clone)]
pub struct IngestInboxStartResult {
    pub job_id: String,
    pub doc_id: String,
}

fn jobs_set() -> &'static Mutex<BTreeSet<String>> {
    ACTIVE_JOBS.get_or_init(|| Mutex::new(BTreeSet::new()))
}

fn mime_for_path(path: &Path) -> String {
    match path.extension().and_then(|x| x.to_str()).unwrap_or_default() {
        "md" => "text/markdown".to_string(),
        "html" | "htm" => "text/html".to_string(),
        "pdf" => "application/pdf".to_string(),
        "txt" => "text/plain".to_string(),
        _ => "application/octet-stream".to_string(),
    }
}

pub fn vault_init_service(vault_path: &Path, vault_slug: &str, now_ms: i64) -> AppResult<String> {
    let vault = vault_init(vault_path, vault_slug, now_ms)?;
    Ok(vault.vault_id)
}

pub fn vault_open_service(vault_path: &Path) -> AppResult<VaultSummary> {
    let vault = vault_open(vault_path)?;
    Ok(VaultSummary {
        vault_id: vault.vault_id,
        vault_slug: vault.vault_slug,
    })
}

pub fn ingest_scan_folder_service(
    vault_path: &Path,
    scan_root: &Path,
    source_kind: &str,
    now_ms: i64,
) -> AppResult<i64> {
    let vault = vault_open(vault_path)?;
    let conn = open_db(&vault_path.join(vault.db.relative_path))?;
    let store = ObjectStore::new(vault_paths(vault_path).objects_dir);

    let mut files: Vec<PathBuf> = walkdir::WalkDir::new(scan_root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .collect();
    files.sort();

    let mut ingested = 0i64;
    for path in files {
        let bytes = fs::read(&path).map_err(|e| {
            AppError::new(
                "KC_INGEST_FAILED",
                "ingest",
                "failed reading scan file",
                true,
                serde_json::json!({ "error": e.to_string(), "path": path }),
            )
        })?;
        ingest_bytes(
            &conn,
            &store,
            &bytes,
            &mime_for_path(&path),
            source_kind,
            now_ms,
            Some(&path.to_string_lossy()),
            now_ms,
        )?;
        ingested += 1;
    }

    Ok(ingested)
}

pub fn ingest_inbox_start_service(
    vault_path: &Path,
    file_path: &Path,
    source_kind: &str,
    now_ms: i64,
) -> AppResult<IngestInboxStartResult> {
    let vault = vault_open(vault_path)?;
    let conn = open_db(&vault_path.join(vault.db.relative_path))?;
    let store = ObjectStore::new(vault_paths(vault_path).objects_dir);

    let bytes = fs::read(file_path).map_err(|e| {
        AppError::new(
            "KC_INGEST_FAILED",
            "ingest",
            "failed reading inbox file",
            true,
            serde_json::json!({ "error": e.to_string(), "path": file_path }),
        )
    })?;

    let out = ingest_bytes(
        &conn,
        &store,
        &bytes,
        &mime_for_path(file_path),
        source_kind,
        now_ms,
        Some(&file_path.to_string_lossy()),
        now_ms,
    )?;

    let job_id = format!(
        "inbox:{}",
        blake3_hex_prefixed(format!("{}\n{}", out.doc_id.0, now_ms).as_bytes())
    );
    let mut jobs = jobs_set().lock().map_err(|_| {
        AppError::new(
            "KC_INTERNAL_ERROR",
            "jobs",
            "failed acquiring active jobs lock",
            true,
            serde_json::json!({}),
        )
    })?;
    jobs.insert(job_id.clone());

    Ok(IngestInboxStartResult {
        job_id,
        doc_id: out.doc_id.0,
    })
}

pub fn ingest_inbox_stop_service(job_id: &str) -> AppResult<bool> {
    let mut jobs = jobs_set().lock().map_err(|_| {
        AppError::new(
            "KC_INTERNAL_ERROR",
            "jobs",
            "failed acquiring active jobs lock",
            true,
            serde_json::json!({}),
        )
    })?;
    Ok(jobs.remove(job_id))
}

pub fn search_query_service(
    vault_path: &Path,
    query: &str,
    _now_ms: i64,
    limit: usize,
) -> AppResult<Vec<SearchHit>> {
    let vault = vault_open(vault_path)?;
    let conn = open_db(&vault_path.join(vault.db.relative_path))?;
    let store = ObjectStore::new(vault_paths(vault_path).objects_dir);
    let mut stmt = conn.prepare(
        "SELECT doc_id FROM canonical_text ORDER BY created_event_id DESC, doc_id ASC LIMIT ?1",
    ).map_err(|e| {
        AppError::new(
            "KC_RETRIEVAL_FAILED",
            "search",
            "failed preparing search query",
            false,
            serde_json::json!({ "error": e.to_string() }),
        )
    })?;
    let rows = stmt
        .query_map([limit as i64], |row| row.get::<_, String>(0))
        .map_err(|e| {
            AppError::new(
                "KC_RETRIEVAL_FAILED",
                "search",
                "failed running search query",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;

    let query_lower = query.to_lowercase();
    let mut hits = Vec::new();
    for row in rows {
        let doc_id = row.map_err(|e| {
            AppError::new(
                "KC_RETRIEVAL_FAILED",
                "search",
                "failed reading search row",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;
        let text =
            String::from_utf8(load_canonical_text(&conn, &store, &DocId(doc_id.clone()))?).unwrap_or_default();
        if text.to_lowercase().contains(&query_lower) {
            hits.push(SearchHit {
                doc_id,
                score: 1.0,
                snippet: text.chars().take(120).collect(),
            });
        }
    }
    Ok(hits)
}

pub fn locator_resolve_service(vault_path: &Path, locator: &LocatorV1) -> AppResult<String> {
    let vault = vault_open(vault_path)?;
    let conn = open_db(&vault_path.join(vault.db.relative_path))?;
    let store = ObjectStore::new(vault_paths(vault_path).objects_dir);
    resolve_locator_strict(&conn, &store, locator)
}

pub fn export_bundle_service(
    vault_path: &Path,
    export_dir: &Path,
    include_vectors: bool,
    now_ms: i64,
) -> AppResult<PathBuf> {
    crate::export::export_bundle(
        vault_path,
        export_dir,
        &crate::export::ExportOptions { include_vectors },
        now_ms,
    )
}

pub fn events_list_service(vault_path: &Path, limit: i64) -> AppResult<Vec<EventItem>> {
    let vault = vault_open(vault_path)?;
    let conn = open_db(&vault_path.join(vault.db.relative_path))?;
    let mut stmt = conn
        .prepare("SELECT event_id, ts_ms, type FROM events ORDER BY event_id DESC LIMIT ?1")
        .map_err(|e| {
            AppError::new(
                "KC_DB_INTEGRITY_FAILED",
                "events",
                "failed preparing events query",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;
    let rows = stmt
        .query_map([limit.max(1)], |row| {
            Ok(EventItem {
                event_id: row.get(0)?,
                ts_ms: row.get(1)?,
                event_type: row.get(2)?,
            })
        })
        .map_err(|e| {
            AppError::new(
                "KC_DB_INTEGRITY_FAILED",
                "events",
                "failed querying events",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;

    let mut events = Vec::new();
    for row in rows {
        events.push(row.map_err(|e| {
            AppError::new(
                "KC_DB_INTEGRITY_FAILED",
                "events",
                "failed decoding event row",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?);
    }
    Ok(events)
}

pub fn jobs_list_service(_vault_path: &Path) -> AppResult<Vec<String>> {
    let jobs = jobs_set().lock().map_err(|_| {
        AppError::new(
            "KC_INTERNAL_ERROR",
            "jobs",
            "failed acquiring active jobs lock",
            true,
            serde_json::json!({}),
        )
    })?;
    Ok(jobs.iter().cloned().collect())
}

pub fn rpc_health_snapshot_service(vault_path: &Path) -> AppResult<serde_json::Value> {
    let vault = vault_open(vault_path)?;
    let conn = open_db(&vault_path.join(vault.db.relative_path.clone()))?;
    let db_bytes = fs::read(vault_path.join(vault.db.relative_path)).map_err(|e| {
        AppError::new(
            "KC_RPC_FAILED",
            "rpc",
            "failed reading db for health snapshot",
            false,
            serde_json::json!({ "error": e.to_string() }),
        )
    })?;
    let event_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM events", [], |r| r.get(0))
        .unwrap_or(0);
    Ok(serde_json::json!({
        "vaultId": vault.vault_id,
        "dbHash": blake3_hex_prefixed(&db_bytes),
        "eventCount": event_count
    }))
}
