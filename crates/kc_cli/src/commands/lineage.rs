use kc_core::app_error::AppResult;
use kc_core::db::open_db;
use kc_core::lineage::{
    lineage_lock_acquire, lineage_lock_release, lineage_lock_status, lineage_overlay_add,
    lineage_overlay_list, lineage_overlay_remove,
};
use kc_core::lineage_governance::{
    lineage_lock_acquire_scope, lineage_role_grant, lineage_role_list, lineage_role_revoke,
};
use kc_core::vault::vault_open;
use std::path::Path;

pub fn run_overlay_add(
    vault_path: &str,
    doc_id: &str,
    from_node_id: &str,
    to_node_id: &str,
    relation: &str,
    evidence: &str,
    lock_token: &str,
    created_by: &str,
    now_ms: i64,
) -> AppResult<()> {
    let vault = vault_open(Path::new(vault_path))?;
    let conn = open_db(&Path::new(vault_path).join(vault.db.relative_path))?;
    let entry = lineage_overlay_add(
        &conn,
        doc_id,
        from_node_id,
        to_node_id,
        relation,
        evidence,
        lock_token,
        now_ms,
        created_by,
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "status": "ok",
            "overlay": entry
        }))
        .unwrap_or_else(|_| "{}".to_string())
    );
    Ok(())
}

pub fn run_overlay_remove(
    vault_path: &str,
    overlay_id: &str,
    lock_token: &str,
    now_ms: i64,
) -> AppResult<()> {
    let vault = vault_open(Path::new(vault_path))?;
    let conn = open_db(&Path::new(vault_path).join(vault.db.relative_path))?;
    lineage_overlay_remove(&conn, overlay_id, lock_token, now_ms)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "status": "ok",
            "removed_overlay_id": overlay_id
        }))
        .unwrap_or_else(|_| "{}".to_string())
    );
    Ok(())
}

pub fn run_overlay_list(vault_path: &str, doc_id: &str) -> AppResult<()> {
    let vault = vault_open(Path::new(vault_path))?;
    let conn = open_db(&Path::new(vault_path).join(vault.db.relative_path))?;
    let overlays = lineage_overlay_list(&conn, doc_id)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "status": "ok",
            "doc_id": doc_id,
            "overlays": overlays
        }))
        .unwrap_or_else(|_| "{}".to_string())
    );
    Ok(())
}

pub fn run_role_grant(
    vault_path: &str,
    subject: &str,
    role: &str,
    granted_by: &str,
    now_ms: i64,
) -> AppResult<()> {
    let vault = vault_open(Path::new(vault_path))?;
    let conn = open_db(&Path::new(vault_path).join(vault.db.relative_path))?;
    let binding = lineage_role_grant(&conn, subject, role, granted_by, now_ms)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "status": "ok",
            "binding": binding
        }))
        .unwrap_or_else(|_| "{}".to_string())
    );
    Ok(())
}

pub fn run_role_revoke(vault_path: &str, subject: &str, role: &str) -> AppResult<()> {
    let vault = vault_open(Path::new(vault_path))?;
    let conn = open_db(&Path::new(vault_path).join(vault.db.relative_path))?;
    lineage_role_revoke(&conn, subject, role)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "status": "ok",
            "revoked": true,
            "subject": subject,
            "role": role
        }))
        .unwrap_or_else(|_| "{}".to_string())
    );
    Ok(())
}

pub fn run_role_list(vault_path: &str) -> AppResult<()> {
    let vault = vault_open(Path::new(vault_path))?;
    let conn = open_db(&Path::new(vault_path).join(vault.db.relative_path))?;
    let bindings = lineage_role_list(&conn)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "status": "ok",
            "bindings": bindings
        }))
        .unwrap_or_else(|_| "{}".to_string())
    );
    Ok(())
}

pub fn run_lock_acquire(vault_path: &str, doc_id: &str, owner: &str, now_ms: i64) -> AppResult<()> {
    let vault = vault_open(Path::new(vault_path))?;
    let conn = open_db(&Path::new(vault_path).join(vault.db.relative_path))?;
    let lease = lineage_lock_acquire(&conn, doc_id, owner, now_ms)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "status": "ok",
            "lease": lease
        }))
        .unwrap_or_else(|_| "{}".to_string())
    );
    Ok(())
}

pub fn run_lock_acquire_scope(
    vault_path: &str,
    scope_kind: &str,
    scope_value: &str,
    owner: &str,
    now_ms: i64,
) -> AppResult<()> {
    let vault = vault_open(Path::new(vault_path))?;
    let conn = open_db(&Path::new(vault_path).join(vault.db.relative_path))?;
    let lease = lineage_lock_acquire_scope(&conn, scope_kind, scope_value, owner, now_ms)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "status": "ok",
            "lease": lease
        }))
        .unwrap_or_else(|_| "{}".to_string())
    );
    Ok(())
}

pub fn run_lock_release(vault_path: &str, doc_id: &str, token: &str) -> AppResult<()> {
    let vault = vault_open(Path::new(vault_path))?;
    let conn = open_db(&Path::new(vault_path).join(vault.db.relative_path))?;
    lineage_lock_release(&conn, doc_id, token)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "status": "ok",
            "released": true,
            "doc_id": doc_id
        }))
        .unwrap_or_else(|_| "{}".to_string())
    );
    Ok(())
}

pub fn run_lock_status(vault_path: &str, doc_id: &str, now_ms: i64) -> AppResult<()> {
    let vault = vault_open(Path::new(vault_path))?;
    let conn = open_db(&Path::new(vault_path).join(vault.db.relative_path))?;
    let status = lineage_lock_status(&conn, doc_id, now_ms)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "status": "ok",
            "lock": status
        }))
        .unwrap_or_else(|_| "{}".to_string())
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        run_lock_acquire, run_lock_acquire_scope, run_lock_release, run_lock_status,
        run_overlay_add, run_overlay_list, run_overlay_remove, run_role_grant, run_role_list,
        run_role_revoke,
    };
    use kc_core::db::open_db;
    use kc_core::ingest::ingest_bytes;
    use kc_core::lineage::{lineage_lock_acquire, lineage_overlay_list};
    use kc_core::lineage_governance::{
        lineage_lock_scope_status, lineage_role_grant, lineage_role_list,
    };
    use kc_core::lineage_policy::{lineage_policy_add, lineage_policy_bind};
    use kc_core::object_store::ObjectStore;
    use kc_core::vault::vault_init;

    #[test]
    fn overlay_commands_round_trip() {
        let root = tempfile::tempdir().expect("tempdir").keep();
        vault_init(&root, "demo", 1).expect("vault init");

        let conn = open_db(&root.join("db/knowledge.sqlite")).expect("open db");
        let store = ObjectStore::new(root.join("store/objects"));
        let ingested = ingest_bytes(
            &conn,
            &store,
            b"cli lineage overlay",
            "text/plain",
            "notes",
            1,
            None,
            1,
        )
        .expect("ingest");

        let doc_id = ingested.doc_id.0.clone();
        let doc_node = format!("doc:{}", doc_id);
        let chunk_node = "chunk:cli-overlay";
        lineage_role_grant(&conn, "cli-test", "editor", "test-harness", 2).expect("grant role");
        lineage_policy_add(
            &conn,
            "allow-overlay-cli",
            "allow",
            r#"{"action":"lineage.overlay.write"}"#,
            "test-harness",
            2,
        )
        .expect("add policy");
        lineage_policy_bind(&conn, "cli-test", "allow-overlay-cli", "test-harness", 2)
            .expect("bind policy");
        let lock = lineage_lock_acquire(&conn, &doc_id, "cli-test", 2).expect("acquire lock");
        let lock_token = lock.token.clone();
        drop(conn);

        run_overlay_add(
            root.to_string_lossy().as_ref(),
            &doc_id,
            &doc_node,
            chunk_node,
            "related_to",
            "cli",
            &lock_token,
            "cli-test",
            3,
        )
        .expect("overlay add");
        run_overlay_list(root.to_string_lossy().as_ref(), &doc_id).expect("overlay list");

        let conn = open_db(&root.join("db/knowledge.sqlite")).expect("open db 2");
        let listed = lineage_overlay_list(&conn, &doc_id).expect("overlay list direct");
        assert_eq!(listed.len(), 1);
        let overlay_id = listed[0].overlay_id.clone();
        drop(conn);

        run_overlay_remove(root.to_string_lossy().as_ref(), &overlay_id, &lock_token, 4)
            .expect("overlay remove");

        run_lock_status(root.to_string_lossy().as_ref(), &doc_id, 5).expect("lock status");
        run_lock_release(root.to_string_lossy().as_ref(), &doc_id, &lock_token)
            .expect("lock release");
        run_lock_acquire(root.to_string_lossy().as_ref(), &doc_id, "cli-test", 6)
            .expect("lock acquire");
    }

    #[test]
    fn role_and_scope_commands_round_trip() {
        let root = tempfile::tempdir().expect("tempdir").keep();
        vault_init(&root, "demo", 1).expect("vault init");

        run_role_grant(
            root.to_string_lossy().as_ref(),
            "subject-a",
            "editor",
            "cli-test",
            10,
        )
        .expect("role grant");
        run_role_list(root.to_string_lossy().as_ref()).expect("role list");

        let conn = open_db(&root.join("db/knowledge.sqlite")).expect("open db");
        let listed = lineage_role_list(&conn).expect("direct role list");
        assert!(listed
            .iter()
            .any(|binding| binding.subject_id == "subject-a" && binding.role_name == "editor"));
        drop(conn);

        run_lock_acquire_scope(
            root.to_string_lossy().as_ref(),
            "doc",
            "doc-scope",
            "scope-owner",
            11,
        )
        .expect("acquire scope");

        let conn = open_db(&root.join("db/knowledge.sqlite")).expect("open db");
        let status =
            lineage_lock_scope_status(&conn, "doc", "doc-scope", 12).expect("scope status");
        assert!(status.held);
        assert_eq!(status.owner.as_deref(), Some("scope-owner"));
        drop(conn);

        run_role_revoke(root.to_string_lossy().as_ref(), "subject-a", "editor")
            .expect("role revoke");
    }
}
