use crate::app_error::{AppError, AppResult};
use crate::canon_json::to_canonical_bytes;
use crate::db::open_db;
use crate::export::{export_bundle, ExportOptions};
use crate::hashing::blake3_hex_prefixed;
use crate::vault::vault_open;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncHeadV1 {
    pub schema_version: i64,
    pub snapshot_id: String,
    pub manifest_hash: String,
    pub created_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatusV1 {
    pub target_path: String,
    pub remote_head: Option<SyncHeadV1>,
    pub seen_remote_snapshot_id: Option<String>,
    pub last_applied_manifest_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPushResultV1 {
    pub snapshot_id: String,
    pub manifest_hash: String,
    pub remote_head: SyncHeadV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPullResultV1 {
    pub snapshot_id: String,
    pub manifest_hash: String,
    pub remote_head: SyncHeadV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConflictArtifactV1 {
    pub schema_version: i64,
    pub kind: String,
    pub vault_id: String,
    pub now_ms: i64,
    pub local_manifest_hash: String,
    pub remote_head_snapshot_id: Option<String>,
    pub remote_head_manifest_hash: Option<String>,
    pub seen_remote_snapshot_id: Option<String>,
}

fn sync_error(code: &str, message: &str, details: serde_json::Value) -> AppError {
    AppError::new(code, "sync", message, false, details)
}

fn ensure_sync_tables(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS sync_state (
           state_key TEXT PRIMARY KEY,
           state_value TEXT NOT NULL,
           updated_at_ms INTEGER NOT NULL
         );
         CREATE TABLE IF NOT EXISTS sync_snapshots (
           snapshot_id TEXT PRIMARY KEY,
           direction TEXT NOT NULL,
           created_at_ms INTEGER NOT NULL,
           bundle_relpath TEXT NOT NULL,
           manifest_hash TEXT NOT NULL
         );",
    )
    .map_err(|e| {
        sync_error(
            "KC_SYNC_STATE_FAILED",
            "failed ensuring sync tables",
            serde_json::json!({ "error": e.to_string() }),
        )
    })
}

fn read_state(conn: &Connection, key: &str) -> AppResult<Option<String>> {
    let mut stmt = conn
        .prepare("SELECT state_value FROM sync_state WHERE state_key=?1")
        .map_err(|e| {
            sync_error(
                "KC_SYNC_STATE_FAILED",
                "failed preparing sync state query",
                serde_json::json!({ "error": e.to_string(), "key": key }),
            )
        })?;
    let mut rows = stmt.query([key]).map_err(|e| {
        sync_error(
            "KC_SYNC_STATE_FAILED",
            "failed querying sync state",
            serde_json::json!({ "error": e.to_string(), "key": key }),
        )
    })?;
    let value = rows.next().map_err(|e| {
        sync_error(
            "KC_SYNC_STATE_FAILED",
            "failed iterating sync state row",
            serde_json::json!({ "error": e.to_string(), "key": key }),
        )
    })?;
    Ok(value.map(|row| row.get(0)).transpose().map_err(|e| {
        sync_error(
            "KC_SYNC_STATE_FAILED",
            "failed decoding sync state row",
            serde_json::json!({ "error": e.to_string(), "key": key }),
        )
    })?)
}

fn write_state(conn: &Connection, key: &str, value: &str, now_ms: i64) -> AppResult<()> {
    conn.execute(
        "INSERT INTO sync_state(state_key, state_value, updated_at_ms)
         VALUES(?1, ?2, ?3)
         ON CONFLICT(state_key) DO UPDATE SET state_value=excluded.state_value, updated_at_ms=excluded.updated_at_ms",
        params![key, value, now_ms],
    )
    .map_err(|e| {
        sync_error(
            "KC_SYNC_STATE_FAILED",
            "failed writing sync state",
            serde_json::json!({ "error": e.to_string(), "key": key }),
        )
    })?;
    Ok(())
}

fn write_snapshot_log(
    conn: &Connection,
    snapshot_id: &str,
    direction: &str,
    created_at_ms: i64,
    bundle_relpath: &str,
    manifest_hash: &str,
) -> AppResult<()> {
    conn.execute(
        "INSERT OR REPLACE INTO sync_snapshots(snapshot_id, direction, created_at_ms, bundle_relpath, manifest_hash)
         VALUES(?1, ?2, ?3, ?4, ?5)",
        params![snapshot_id, direction, created_at_ms, bundle_relpath, manifest_hash],
    )
    .map_err(|e| {
        sync_error(
            "KC_SYNC_STATE_FAILED",
            "failed writing sync snapshot log",
            serde_json::json!({ "error": e.to_string(), "snapshot_id": snapshot_id }),
        )
    })?;
    Ok(())
}

fn main_db_path(conn: &Connection) -> AppResult<PathBuf> {
    let path: String = conn
        .query_row(
            "SELECT file FROM pragma_database_list WHERE name='main'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| {
            sync_error(
                "KC_SYNC_STATE_FAILED",
                "failed resolving main database path",
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;
    Ok(PathBuf::from(path))
}

fn ensure_target_layout(target_path: &Path) -> AppResult<(PathBuf, PathBuf)> {
    let snapshots = target_path.join("snapshots");
    let conflicts = target_path.join("conflicts");
    fs::create_dir_all(&snapshots).map_err(|e| {
        sync_error(
            "KC_SYNC_TARGET_INVALID",
            "failed creating snapshots directory",
            serde_json::json!({ "error": e.to_string(), "path": snapshots }),
        )
    })?;
    fs::create_dir_all(&conflicts).map_err(|e| {
        sync_error(
            "KC_SYNC_TARGET_INVALID",
            "failed creating conflicts directory",
            serde_json::json!({ "error": e.to_string(), "path": conflicts }),
        )
    })?;
    Ok((snapshots, conflicts))
}

fn read_head(target_path: &Path) -> AppResult<Option<SyncHeadV1>> {
    let head_path = target_path.join("head.json");
    if !head_path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(&head_path).map_err(|e| {
        sync_error(
            "KC_SYNC_TARGET_INVALID",
            "failed reading sync head file",
            serde_json::json!({ "error": e.to_string(), "path": head_path }),
        )
    })?;
    serde_json::from_slice(&bytes).map(Some).map_err(|e| {
        sync_error(
            "KC_SYNC_TARGET_INVALID",
            "failed parsing sync head file",
            serde_json::json!({ "error": e.to_string(), "path": head_path }),
        )
    })
}

fn write_head(target_path: &Path, head: &SyncHeadV1) -> AppResult<()> {
    let bytes = to_canonical_bytes(&serde_json::to_value(head).map_err(|e| {
        sync_error(
            "KC_SYNC_TARGET_INVALID",
            "failed serializing sync head",
            serde_json::json!({ "error": e.to_string() }),
        )
    })?)?;
    fs::write(target_path.join("head.json"), bytes).map_err(|e| {
        sync_error(
            "KC_SYNC_TARGET_INVALID",
            "failed writing sync head",
            serde_json::json!({ "error": e.to_string(), "path": target_path.join("head.json") }),
        )
    })?;
    Ok(())
}

fn list_files_sorted(root: &Path) -> AppResult<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .collect();
    files.sort();
    Ok(files)
}

fn copy_tree_sorted(src: &Path, dst: &Path) -> AppResult<()> {
    for file in list_files_sorted(src)? {
        let rel = file.strip_prefix(src).map_err(|e| {
            sync_error(
                "KC_SYNC_TARGET_INVALID",
                "failed deriving relative path while copying sync tree",
                serde_json::json!({ "error": e.to_string(), "path": file, "root": src }),
            )
        })?;
        let out = dst.join(rel);
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                sync_error(
                    "KC_SYNC_TARGET_INVALID",
                    "failed creating sync destination parent",
                    serde_json::json!({ "error": e.to_string(), "path": parent }),
                )
            })?;
        }
        fs::copy(&file, &out).map_err(|e| {
            sync_error(
                "KC_SYNC_TARGET_INVALID",
                "failed copying sync file",
                serde_json::json!({ "error": e.to_string(), "from": file, "to": out }),
            )
        })?;
    }
    Ok(())
}

fn compute_manifest_hash(bundle_dir: &Path) -> AppResult<String> {
    let bytes = fs::read(bundle_dir.join("manifest.json")).map_err(|e| {
        sync_error(
            "KC_SYNC_STATE_FAILED",
            "failed reading manifest for sync hash",
            serde_json::json!({ "error": e.to_string(), "path": bundle_dir.join("manifest.json") }),
        )
    })?;
    Ok(blake3_hex_prefixed(&bytes))
}

fn build_local_snapshot(vault_path: &Path, now_ms: i64) -> AppResult<(PathBuf, String)> {
    let staging_root = std::env::temp_dir().join(format!(
        "kc_sync_export_{}_{}",
        std::process::id(),
        now_ms
    ));
    fs::create_dir_all(&staging_root).map_err(|e| {
        sync_error(
            "KC_SYNC_STATE_FAILED",
            "failed creating local sync staging directory",
            serde_json::json!({ "error": e.to_string(), "path": staging_root }),
        )
    })?;

    let bundle = export_bundle(
        vault_path,
        &staging_root,
        &ExportOptions {
            include_vectors: true,
            as_zip: false,
        },
        now_ms,
    )?;
    let manifest_hash = compute_manifest_hash(&bundle)?;
    Ok((bundle, manifest_hash))
}

fn create_conflict_artifact(
    target_path: &Path,
    vault_id: &str,
    now_ms: i64,
    local_manifest_hash: &str,
    remote_head: Option<&SyncHeadV1>,
    seen_remote_snapshot_id: Option<String>,
) -> AppResult<PathBuf> {
    let artifact = SyncConflictArtifactV1 {
        schema_version: 1,
        kind: "sync_conflict".to_string(),
        vault_id: vault_id.to_string(),
        now_ms,
        local_manifest_hash: local_manifest_hash.to_string(),
        remote_head_snapshot_id: remote_head.map(|h| h.snapshot_id.clone()),
        remote_head_manifest_hash: remote_head.map(|h| h.manifest_hash.clone()),
        seen_remote_snapshot_id,
    };
    let canonical = to_canonical_bytes(&serde_json::to_value(&artifact).map_err(|e| {
        sync_error(
            "KC_SYNC_CONFLICT",
            "failed serializing sync conflict artifact",
            serde_json::json!({ "error": e.to_string() }),
        )
    })?)?;
    let digest = blake3_hex_prefixed(&canonical);
    let digest = digest.strip_prefix("blake3:").unwrap_or(&digest);
    let id = digest[..digest.len().min(16)].to_string();
    let path = target_path
        .join("conflicts")
        .join(format!("conflict_{}_{}.json", now_ms, id));
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            sync_error(
                "KC_SYNC_CONFLICT",
                "failed creating conflict artifact directory",
                serde_json::json!({ "error": e.to_string(), "path": parent }),
            )
        })?;
    }
    fs::write(&path, canonical).map_err(|e| {
        sync_error(
            "KC_SYNC_CONFLICT",
            "failed writing conflict artifact",
            serde_json::json!({ "error": e.to_string(), "path": path }),
        )
    })?;
    Ok(path)
}

fn should_conflict(
    remote_head: Option<&SyncHeadV1>,
    seen_remote_snapshot_id: Option<&str>,
    last_applied_manifest_hash: Option<&str>,
    local_manifest_hash: &str,
) -> bool {
    let remote_changed = match (remote_head, seen_remote_snapshot_id) {
        (Some(head), Some(seen)) => head.snapshot_id != seen,
        _ => false,
    };
    let local_changed = match last_applied_manifest_hash {
        Some(last) => last != local_manifest_hash,
        None => false,
    };
    remote_changed && local_changed
}

fn apply_snapshot_to_vault(snapshot_dir: &Path, vault_path: &Path) -> AppResult<()> {
    for top in ["db", "store", "index"] {
        let src = snapshot_dir.join(top);
        if !src.exists() {
            continue;
        }
        let dst = vault_path.join(top);
        if dst.exists() {
            if dst.is_dir() {
                fs::remove_dir_all(&dst).map_err(|e| {
                    sync_error(
                        "KC_SYNC_APPLY_FAILED",
                        "failed removing existing sync apply directory",
                        serde_json::json!({ "error": e.to_string(), "path": dst }),
                    )
                })?;
            } else {
                fs::remove_file(&dst).map_err(|e| {
                    sync_error(
                        "KC_SYNC_APPLY_FAILED",
                        "failed removing existing sync apply file",
                        serde_json::json!({ "error": e.to_string(), "path": dst }),
                    )
                })?;
            }
        }
        fs::create_dir_all(&dst).map_err(|e| {
            sync_error(
                "KC_SYNC_APPLY_FAILED",
                "failed creating destination sync apply directory",
                serde_json::json!({ "error": e.to_string(), "path": dst }),
            )
        })?;
        copy_tree_sorted(&src, &dst)?;
    }
    Ok(())
}

pub fn sync_status(
    conn: &Connection,
    target_path: &Path,
) -> AppResult<SyncStatusV1> {
    ensure_sync_tables(conn)?;
    let remote_head = read_head(target_path)?;
    let seen_remote_snapshot_id = read_state(conn, "sync_remote_head_seen")?;
    let last_applied_manifest_hash = read_state(conn, "sync_last_applied_manifest_hash")?;

    Ok(SyncStatusV1 {
        target_path: target_path.display().to_string(),
        remote_head,
        seen_remote_snapshot_id,
        last_applied_manifest_hash,
    })
}

pub fn sync_push(
    conn: &Connection,
    vault_path: &Path,
    target_path: &Path,
    now_ms: i64,
) -> AppResult<SyncPushResultV1> {
    ensure_sync_tables(conn)?;
    let vault = vault_open(vault_path)?;
    let (_snapshots, _conflicts) = ensure_target_layout(target_path)?;
    let remote_head = read_head(target_path)?;

    let seen_remote = read_state(conn, "sync_remote_head_seen")?;
    let last_applied_manifest = read_state(conn, "sync_last_applied_manifest_hash")?;

    let (local_bundle, local_manifest_hash) = build_local_snapshot(vault_path, now_ms)?;

    if should_conflict(
        remote_head.as_ref(),
        seen_remote.as_deref(),
        last_applied_manifest.as_deref(),
        &local_manifest_hash,
    ) {
        let conflict_path = create_conflict_artifact(
            target_path,
            &vault.vault_id,
            now_ms,
            &local_manifest_hash,
            remote_head.as_ref(),
            seen_remote,
        )?;
        return Err(sync_error(
            "KC_SYNC_CONFLICT",
            "remote and local changes diverged; conflict artifact emitted",
            serde_json::json!({ "conflict_artifact": conflict_path }),
        ));
    }

    let snapshot_id = blake3_hex_prefixed(
        format!("kc.sync.snapshot.v1\n{}\n{}", local_manifest_hash, now_ms).as_bytes(),
    );
    let snapshot_dir = target_path.join("snapshots").join(&snapshot_id);
    if snapshot_dir.exists() {
        fs::remove_dir_all(&snapshot_dir).map_err(|e| {
            sync_error(
                "KC_SYNC_TARGET_INVALID",
                "failed replacing existing snapshot directory",
                serde_json::json!({ "error": e.to_string(), "path": snapshot_dir }),
            )
        })?;
    }
    fs::create_dir_all(&snapshot_dir).map_err(|e| {
        sync_error(
            "KC_SYNC_TARGET_INVALID",
            "failed creating snapshot directory",
            serde_json::json!({ "error": e.to_string(), "path": snapshot_dir }),
        )
    })?;
    copy_tree_sorted(&local_bundle, &snapshot_dir)?;

    let head = SyncHeadV1 {
        schema_version: 1,
        snapshot_id: snapshot_id.clone(),
        manifest_hash: local_manifest_hash.clone(),
        created_at_ms: now_ms,
    };
    write_head(target_path, &head)?;

    write_state(conn, "sync_remote_head_seen", &snapshot_id, now_ms)?;
    write_state(
        conn,
        "sync_last_applied_manifest_hash",
        &local_manifest_hash,
        now_ms,
    )?;
    write_state(conn, "sync_last_applied_snapshot_id", &snapshot_id, now_ms)?;
    write_snapshot_log(
        conn,
        &snapshot_id,
        "push",
        now_ms,
        &format!("snapshots/{}", snapshot_id),
        &local_manifest_hash,
    )?;

    Ok(SyncPushResultV1 {
        snapshot_id,
        manifest_hash: local_manifest_hash,
        remote_head: head,
    })
}

pub fn sync_pull(
    conn: &Connection,
    vault_path: &Path,
    target_path: &Path,
    now_ms: i64,
) -> AppResult<SyncPullResultV1> {
    ensure_sync_tables(conn)?;
    let db_path = main_db_path(conn)?;
    let vault = vault_open(vault_path)?;
    let remote_head = read_head(target_path)?.ok_or_else(|| {
        sync_error(
            "KC_SYNC_TARGET_INVALID",
            "sync target has no head.json",
            serde_json::json!({ "target_path": target_path }),
        )
    })?;

    let seen_remote = read_state(conn, "sync_remote_head_seen")?;
    let last_applied_manifest = read_state(conn, "sync_last_applied_manifest_hash")?;

    let (_local_bundle, local_manifest_hash) = build_local_snapshot(vault_path, now_ms)?;

    if should_conflict(
        Some(&remote_head),
        seen_remote.as_deref(),
        last_applied_manifest.as_deref(),
        &local_manifest_hash,
    ) {
        let conflict_path = create_conflict_artifact(
            target_path,
            &vault.vault_id,
            now_ms,
            &local_manifest_hash,
            Some(&remote_head),
            seen_remote,
        )?;
        return Err(sync_error(
            "KC_SYNC_CONFLICT",
            "remote and local changes diverged; conflict artifact emitted",
            serde_json::json!({ "conflict_artifact": conflict_path }),
        ));
    }

    let snapshot_dir = target_path.join("snapshots").join(&remote_head.snapshot_id);
    if !snapshot_dir.exists() {
        return Err(sync_error(
            "KC_SYNC_TARGET_INVALID",
            "remote snapshot directory missing for head",
            serde_json::json!({
                "snapshot_id": remote_head.snapshot_id,
                "path": snapshot_dir
            }),
        ));
    }

    apply_snapshot_to_vault(&snapshot_dir, vault_path)?;

    let post_conn = open_db(&db_path)?;
    write_state(
        &post_conn,
        "sync_remote_head_seen",
        &remote_head.snapshot_id,
        now_ms,
    )?;
    write_state(
        &post_conn,
        "sync_last_applied_manifest_hash",
        &remote_head.manifest_hash,
        now_ms,
    )?;
    write_state(
        &post_conn,
        "sync_last_applied_snapshot_id",
        &remote_head.snapshot_id,
        now_ms,
    )?;
    write_snapshot_log(
        &post_conn,
        &remote_head.snapshot_id,
        "pull",
        now_ms,
        &format!("snapshots/{}", remote_head.snapshot_id),
        &remote_head.manifest_hash,
    )?;

    Ok(SyncPullResultV1 {
        snapshot_id: remote_head.snapshot_id.clone(),
        manifest_hash: remote_head.manifest_hash.clone(),
        remote_head,
    })
}
