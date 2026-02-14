use crate::app_error::{AppError, AppResult};
use crate::canonical::load_canonical_text;
use crate::db::{
    db_is_unlocked, db_lock, db_unlock, migrate_db_to_sqlcipher, open_db, DbMigrationOutcome,
};
use crate::events::append_event;
use crate::hashing::blake3_hex_prefixed;
use crate::ingest::ingest_bytes;
use crate::locator::{resolve_locator_strict, LocatorV1};
use crate::object_store::{is_encrypted_payload, ObjectStore};
use crate::recovery::{generate_recovery_bundle, verify_recovery_bundle, RecoveryManifestV1};
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

#[derive(Debug, Clone)]
pub struct VaultRecoveryStatus {
    pub vault_id: String,
    pub encryption_enabled: bool,
    pub last_bundle_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct VaultRecoveryGenerateResult {
    pub bundle_path: PathBuf,
    pub recovery_phrase: String,
    pub manifest: RecoveryManifestV1,
}

#[derive(Debug, Clone)]
pub struct VaultRecoveryVerifyResult {
    pub manifest: RecoveryManifestV1,
}

#[derive(Debug, Clone)]
pub struct VaultDbLockStatus {
    pub db_encryption_enabled: bool,
    pub unlocked: bool,
    pub mode: String,
    pub key_reference: Option<String>,
}

#[derive(Debug, Clone)]
pub struct VaultDbEncryptStatus {
    pub enabled: bool,
    pub mode: String,
    pub key_reference: Option<String>,
    pub unlocked: bool,
}

#[derive(Debug, Clone)]
pub struct VaultDbEncryptMigrateResult {
    pub status: VaultDbEncryptStatus,
    pub outcome: String,
    pub event_id: i64,
}

fn jobs_set() -> &'static Mutex<BTreeSet<String>> {
    ACTIVE_JOBS.get_or_init(|| Mutex::new(BTreeSet::new()))
}

fn recovery_state_file(vault_path: &Path) -> PathBuf {
    vault_path.join(".kc_recovery_last_path")
}

fn write_recovery_state_file(vault_path: &Path, bundle_path: &Path) -> AppResult<()> {
    fs::write(
        recovery_state_file(vault_path),
        bundle_path.display().to_string(),
    )
    .map_err(|e| {
        AppError::new(
            "KC_RECOVERY_BUNDLE_INVALID",
            "recovery",
            "failed writing recovery state marker",
            false,
            serde_json::json!({ "error": e.to_string(), "vault_path": vault_path }),
        )
    })
}

fn read_recovery_state_file(vault_path: &Path) -> Option<String> {
    let path = recovery_state_file(vault_path);
    let Ok(value) = fs::read_to_string(path) else {
        return None;
    };
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn mime_for_path(path: &Path) -> String {
    match path
        .extension()
        .and_then(|x| x.to_str())
        .unwrap_or_default()
    {
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
    let mut stmt = conn
        .prepare(
            "SELECT doc_id FROM canonical_text ORDER BY created_event_id DESC, doc_id ASC LIMIT ?1",
        )
        .map_err(|e| {
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
        let text = String::from_utf8(load_canonical_text(&conn, &store, &DocId(doc_id.clone()))?)
            .unwrap_or_default();
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
        &crate::export::ExportOptions {
            include_vectors,
            as_zip: false,
        },
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

pub fn sync_status_service(
    vault_path: &Path,
    target_uri: &str,
) -> AppResult<crate::sync::SyncStatusV1> {
    let vault = vault_open(vault_path)?;
    let conn = open_db(&vault_path.join(vault.db.relative_path))?;
    crate::sync::sync_status_target(&conn, target_uri)
}

pub fn sync_push_service(
    vault_path: &Path,
    target_uri: &str,
    now_ms: i64,
) -> AppResult<crate::sync::SyncPushResultV1> {
    let vault = vault_open(vault_path)?;
    let conn = open_db(&vault_path.join(vault.db.relative_path))?;
    crate::sync::sync_push_target(&conn, vault_path, target_uri, now_ms)
}

pub fn sync_pull_service(
    vault_path: &Path,
    target_uri: &str,
    now_ms: i64,
    auto_merge_mode: Option<&str>,
) -> AppResult<crate::sync::SyncPullResultV1> {
    let vault = vault_open(vault_path)?;
    let conn = open_db(&vault_path.join(vault.db.relative_path))?;
    crate::sync::sync_pull_target_with_mode(&conn, vault_path, target_uri, now_ms, auto_merge_mode)
}

pub fn sync_merge_preview_service(
    vault_path: &Path,
    target_uri: &str,
    now_ms: i64,
) -> AppResult<crate::sync::SyncMergePreviewResultV1> {
    let vault = vault_open(vault_path)?;
    let conn = open_db(&vault_path.join(vault.db.relative_path))?;
    crate::sync::sync_merge_preview_target(&conn, vault_path, target_uri, now_ms)
}

pub fn lineage_query_service(
    vault_path: &Path,
    seed_doc_id: &str,
    depth: i64,
    now_ms: i64,
) -> AppResult<crate::lineage::LineageQueryResV1> {
    let vault = vault_open(vault_path)?;
    let conn = open_db(&vault_path.join(vault.db.relative_path))?;
    crate::lineage::query_lineage(&conn, seed_doc_id, depth, now_ms)
}

pub fn lineage_query_v2_service(
    vault_path: &Path,
    seed_doc_id: &str,
    depth: i64,
    now_ms: i64,
) -> AppResult<crate::lineage::LineageQueryResV2> {
    let vault = vault_open(vault_path)?;
    let conn = open_db(&vault_path.join(vault.db.relative_path))?;
    crate::lineage::query_lineage_v2(&conn, seed_doc_id, depth, now_ms)
}

pub fn lineage_overlay_add_service(
    vault_path: &Path,
    doc_id: &str,
    from_node_id: &str,
    to_node_id: &str,
    relation: &str,
    evidence: &str,
    created_at_ms: i64,
    created_by: Option<&str>,
) -> AppResult<crate::lineage::LineageOverlayEntryV1> {
    let vault = vault_open(vault_path)?;
    let conn = open_db(&vault_path.join(vault.db.relative_path))?;
    crate::lineage::lineage_overlay_add(
        &conn,
        doc_id,
        from_node_id,
        to_node_id,
        relation,
        evidence,
        created_at_ms,
        created_by.unwrap_or("overlay"),
    )
}

pub fn lineage_overlay_remove_service(vault_path: &Path, overlay_id: &str) -> AppResult<()> {
    let vault = vault_open(vault_path)?;
    let conn = open_db(&vault_path.join(vault.db.relative_path))?;
    crate::lineage::lineage_overlay_remove(&conn, overlay_id)
}

pub fn lineage_overlay_list_service(
    vault_path: &Path,
    doc_id: &str,
) -> AppResult<Vec<crate::lineage::LineageOverlayEntryV1>> {
    let vault = vault_open(vault_path)?;
    let conn = open_db(&vault_path.join(vault.db.relative_path))?;
    crate::lineage::lineage_overlay_list(&conn, doc_id)
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
    let encrypted_store =
        ObjectStore::with_encryption(vault_paths(vault_path).objects_dir, enc_ctx);

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

pub fn vault_recovery_status_service(vault_path: &Path) -> AppResult<VaultRecoveryStatus> {
    let vault = vault_open(vault_path)?;
    Ok(VaultRecoveryStatus {
        vault_id: vault.vault_id,
        encryption_enabled: vault.encryption.enabled,
        last_bundle_path: read_recovery_state_file(vault_path),
    })
}

pub fn vault_recovery_generate_service(
    vault_path: &Path,
    output_dir: &Path,
    passphrase: &str,
    now_ms: i64,
) -> AppResult<VaultRecoveryGenerateResult> {
    let vault = vault_open(vault_path)?;
    let generated = generate_recovery_bundle(&vault.vault_id, output_dir, passphrase, now_ms)?;
    write_recovery_state_file(vault_path, &generated.bundle_path)?;
    Ok(VaultRecoveryGenerateResult {
        bundle_path: generated.bundle_path,
        recovery_phrase: generated.recovery_phrase,
        manifest: generated.manifest,
    })
}

pub fn vault_recovery_verify_service(
    vault_path: &Path,
    bundle_path: &Path,
    phrase: &str,
) -> AppResult<VaultRecoveryVerifyResult> {
    let vault = vault_open(vault_path)?;
    let manifest = verify_recovery_bundle(&vault.vault_id, bundle_path, phrase)?;
    Ok(VaultRecoveryVerifyResult { manifest })
}

fn db_lock_status_for_vault(
    vault_path: &Path,
    vault: &crate::vault::VaultJsonV2,
) -> VaultDbLockStatus {
    let unlocked = if vault.db_encryption.enabled {
        db_is_unlocked(vault_path)
            || std::env::var("KC_VAULT_DB_PASSPHRASE").is_ok()
            || std::env::var("KC_VAULT_PASSPHRASE").is_ok()
    } else {
        true
    };
    VaultDbLockStatus {
        db_encryption_enabled: vault.db_encryption.enabled,
        unlocked,
        mode: vault.db_encryption.mode.clone(),
        key_reference: vault.db_encryption.key_reference.clone(),
    }
}

fn db_encrypt_status_for_vault(
    vault_path: &Path,
    vault: &crate::vault::VaultJsonV2,
) -> VaultDbEncryptStatus {
    VaultDbEncryptStatus {
        enabled: vault.db_encryption.enabled,
        mode: vault.db_encryption.mode.clone(),
        key_reference: vault.db_encryption.key_reference.clone(),
        unlocked: db_lock_status_for_vault(vault_path, vault).unlocked,
    }
}

fn ensure_passphrase_not_empty(passphrase: &str) -> AppResult<()> {
    if passphrase.is_empty() {
        return Err(AppError::new(
            "KC_DB_KEY_INVALID",
            "db",
            "db passphrase must not be empty",
            false,
            serde_json::json!({}),
        ));
    }
    Ok(())
}

pub fn vault_lock_status_service(vault_path: &Path) -> AppResult<VaultDbLockStatus> {
    let vault = vault_open(vault_path)?;
    Ok(db_lock_status_for_vault(vault_path, &vault))
}

pub fn vault_unlock_service(vault_path: &Path, passphrase: &str) -> AppResult<VaultDbLockStatus> {
    ensure_passphrase_not_empty(passphrase)?;
    let vault = vault_open(vault_path)?;
    if !vault.db_encryption.enabled {
        return Ok(db_lock_status_for_vault(vault_path, &vault));
    }
    let db_path = vault_path.join(vault.db.relative_path.clone());
    db_unlock(vault_path, &db_path, passphrase)?;
    Ok(db_lock_status_for_vault(vault_path, &vault))
}

pub fn vault_lock_service(vault_path: &Path) -> AppResult<VaultDbLockStatus> {
    let vault = vault_open(vault_path)?;
    db_lock(vault_path)?;
    Ok(db_lock_status_for_vault(vault_path, &vault))
}

pub fn vault_db_encrypt_status_service(vault_path: &Path) -> AppResult<VaultDbEncryptStatus> {
    let vault = vault_open(vault_path)?;
    Ok(db_encrypt_status_for_vault(vault_path, &vault))
}

pub fn vault_db_encrypt_enable_service(
    vault_path: &Path,
    passphrase: &str,
) -> AppResult<VaultDbEncryptStatus> {
    ensure_passphrase_not_empty(passphrase)?;
    let mut vault = vault_open(vault_path)?;
    if !vault.db_encryption.enabled {
        vault.db_encryption.enabled = true;
        if vault.db_encryption.key_reference.is_none() {
            vault.db_encryption.key_reference = Some(format!("vaultdb:{}", vault.vault_id));
        }
        vault_save(vault_path, &vault)?;
    }
    let db_path = vault_path.join(vault.db.relative_path.clone());
    db_unlock(vault_path, &db_path, passphrase)?;
    Ok(db_encrypt_status_for_vault(vault_path, &vault))
}

pub fn vault_db_encrypt_migrate_service(
    vault_path: &Path,
    passphrase: &str,
    now_ms: i64,
) -> AppResult<VaultDbEncryptMigrateResult> {
    ensure_passphrase_not_empty(passphrase)?;
    let vault = vault_open(vault_path)?;
    if !vault.db_encryption.enabled {
        return Err(AppError::new(
            "KC_DB_LOCKED",
            "db",
            "db encryption must be enabled before migrate",
            false,
            serde_json::json!({ "vault_path": vault_path }),
        ));
    }
    let db_path = vault_path.join(vault.db.relative_path.clone());
    let migration_outcome = migrate_db_to_sqlcipher(&db_path, passphrase)?;
    db_unlock(vault_path, &db_path, passphrase)?;
    let conn = open_db(&db_path)?;
    let event = append_event(
        &conn,
        now_ms,
        "vault.db_encryption.migrate",
        &serde_json::json!({
            "outcome": match migration_outcome {
                DbMigrationOutcome::Migrated => "migrated",
                DbMigrationOutcome::AlreadyEncrypted => "already_encrypted",
            },
            "mode": vault.db_encryption.mode,
            "kdf_algorithm": vault.db_encryption.kdf.algorithm,
        }),
    )
    .map_err(|e| {
        AppError::new(
            "KC_DB_ENCRYPTION_MIGRATION_FAILED",
            "db",
            "failed appending db encryption migration event",
            false,
            serde_json::json!({ "error": e.code, "message": e.message }),
        )
    })?;
    Ok(VaultDbEncryptMigrateResult {
        status: db_encrypt_status_for_vault(vault_path, &vault),
        outcome: match migration_outcome {
            DbMigrationOutcome::Migrated => "migrated".to_string(),
            DbMigrationOutcome::AlreadyEncrypted => "already_encrypted".to_string(),
        },
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
