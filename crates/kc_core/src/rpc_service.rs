use crate::app_error::{AppError, AppResult};
use crate::canonical::load_canonical_text;
use crate::db::open_db;
use crate::events::append_event;
use crate::hashing::blake3_hex_prefixed;
use crate::ingest::ingest_bytes;
use crate::locator::{resolve_locator_strict, LocatorV1};
use crate::object_store::{is_encrypted_payload, ObjectStore};
use crate::types::{DocId, ObjectHash};
use crate::vault::{vault_init, vault_open, vault_paths, vault_save};
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

#[derive(Debug, Clone)]
pub struct VaultEncryptionStatus {
    pub enabled: bool,
    pub mode: String,
    pub key_reference: Option<String>,
    pub kdf_algorithm: String,
    pub objects_total: i64,
    pub objects_encrypted: i64,
}

#[derive(Debug, Clone)]
pub struct VaultEncryptionMigrateResult {
    pub status: VaultEncryptionStatus,
    pub migrated_objects: i64,
    pub already_encrypted_objects: i64,
    pub event_id: i64,
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

fn object_store_without_passphrase(
    vault: &crate::vault::VaultJsonV2,
    vault_path: &Path,
) -> AppResult<ObjectStore> {
    if vault.encryption_enabled() {
        return Err(AppError::new(
            "KC_ENCRYPTION_REQUIRED",
            "encryption",
            "vault is encrypted; provide passphrase-enabled command path",
            false,
            serde_json::json!({
                "vault_path": vault_path,
                "hint": "use vault encrypt migrate/status flows"
            }),
        ));
    }
    Ok(ObjectStore::new(vault_paths(vault_path).objects_dir))
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
    let conn = open_db(&vault_path.join(vault.db.relative_path.clone()))?;
    let store = object_store_without_passphrase(&vault, vault_path)?;

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
    let conn = open_db(&vault_path.join(vault.db.relative_path.clone()))?;
    let store = object_store_without_passphrase(&vault, vault_path)?;

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
    let conn = open_db(&vault_path.join(vault.db.relative_path.clone()))?;
    let store = object_store_without_passphrase(&vault, vault_path)?;
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
    let conn = open_db(&vault_path.join(vault.db.relative_path.clone()))?;
    let store = object_store_without_passphrase(&vault, vault_path)?;
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

fn load_object_hashes(conn: &rusqlite::Connection) -> AppResult<Vec<ObjectHash>> {
    let mut stmt = conn
        .prepare("SELECT object_hash FROM objects ORDER BY object_hash ASC")
        .map_err(|e| {
            AppError::new(
                "KC_DB_INTEGRITY_FAILED",
                "encryption",
                "failed preparing object hash query",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| {
            AppError::new(
                "KC_DB_INTEGRITY_FAILED",
                "encryption",
                "failed querying object hashes",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;

    let mut hashes = Vec::new();
    for row in rows {
        hashes.push(ObjectHash(row.map_err(|e| {
            AppError::new(
                "KC_DB_INTEGRITY_FAILED",
                "encryption",
                "failed reading object hash row",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?));
    }
    Ok(hashes)
}

fn encryption_status_for_vault(
    vault_path: &Path,
    vault: &crate::vault::VaultJsonV2,
) -> AppResult<VaultEncryptionStatus> {
    let conn = open_db(&vault_path.join(vault.db.relative_path.clone()))?;
    let hashes = load_object_hashes(&conn)?;
    let store = ObjectStore::new(vault_paths(vault_path).objects_dir);
    let mut encrypted = 0i64;
    for hash in &hashes {
        let raw = store.raw_bytes(hash)?;
        if is_encrypted_payload(&raw) {
            encrypted += 1;
        }
    }

    Ok(VaultEncryptionStatus {
        enabled: vault.encryption.enabled,
        mode: vault.encryption.mode.clone(),
        key_reference: vault.encryption.key_reference.clone(),
        kdf_algorithm: vault.encryption.kdf.algorithm.clone(),
        objects_total: hashes.len() as i64,
        objects_encrypted: encrypted,
    })
}

pub fn vault_encryption_status_service(vault_path: &Path) -> AppResult<VaultEncryptionStatus> {
    let vault = vault_open(vault_path)?;
    encryption_status_for_vault(vault_path, &vault)
}

pub fn vault_encryption_enable_service(
    vault_path: &Path,
    passphrase: &str,
) -> AppResult<VaultEncryptionStatus> {
    let mut vault = vault_open(vault_path)?;
    if !vault.encryption.enabled {
        vault.encryption.enabled = true;
        if vault.encryption.key_reference.is_none() {
            vault.encryption.key_reference = Some(format!("vault:{}", vault.vault_id));
        }
    }
    let _ctx = vault.object_store_encryption_context(Some(passphrase))?;
    vault_save(vault_path, &vault)?;
    encryption_status_for_vault(vault_path, &vault)
}

pub fn vault_encryption_migrate_service(
    vault_path: &Path,
    passphrase: &str,
    now_ms: i64,
) -> AppResult<VaultEncryptionMigrateResult> {
    let vault = vault_open(vault_path)?;
    if !vault.encryption.enabled {
        return Err(AppError::new(
            "KC_ENCRYPTION_REQUIRED",
            "encryption",
            "vault encryption must be enabled before migrate",
            false,
            serde_json::json!({ "vault_path": vault_path }),
        ));
    }
    let enc_ctx = vault
        .object_store_encryption_context(Some(passphrase))?
        .ok_or_else(|| {
            AppError::new(
                "KC_ENCRYPTION_REQUIRED",
                "encryption",
                "encryption context unavailable",
                false,
                serde_json::json!({}),
            )
        })?;

    let conn = open_db(&vault_path.join(vault.db.relative_path.clone()))?;
    let hashes = load_object_hashes(&conn)?;
    let plain_store = ObjectStore::new(vault_paths(vault_path).objects_dir.clone());
    let encrypted_store = ObjectStore::with_encryption(vault_paths(vault_path).objects_dir, enc_ctx);

    let mut migrated = 0i64;
    let mut already_encrypted = 0i64;
    for hash in hashes {
        let raw = plain_store.raw_bytes(&hash)?;
        if is_encrypted_payload(&raw) {
            let _ = encrypted_store.get_bytes(&hash)?;
            already_encrypted += 1;
            continue;
        }

        let plaintext = plain_store.get_bytes(&hash)?;
        encrypted_store.rewrite_plaintext_for_hash(&hash, &plaintext)?;
        migrated += 1;
    }

    let event = append_event(
        &conn,
        now_ms,
        "vault.encryption.migrate",
        &serde_json::json!({
            "migrated_objects": migrated,
            "already_encrypted_objects": already_encrypted,
            "mode": vault.encryption.mode,
            "kdf_algorithm": vault.encryption.kdf.algorithm,
        }),
    )
    .map_err(|e| {
        AppError::new(
            "KC_ENCRYPTION_MIGRATION_FAILED",
            "encryption",
            "failed appending migration event",
            false,
            serde_json::json!({ "error": e.code, "message": e.message }),
        )
    })?;

    let status = encryption_status_for_vault(vault_path, &vault)?;
    Ok(VaultEncryptionMigrateResult {
        status,
        migrated_objects: migrated,
        already_encrypted_objects: already_encrypted,
        event_id: event.event_id,
    })
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
