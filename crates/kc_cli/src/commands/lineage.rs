use kc_core::app_error::AppResult;
use kc_core::db::open_db;
use kc_core::lineage::{lineage_overlay_add, lineage_overlay_list, lineage_overlay_remove};
use kc_core::vault::vault_open;
use std::path::Path;

pub fn run_overlay_add(
    vault_path: &str,
    doc_id: &str,
    from_node_id: &str,
    to_node_id: &str,
    relation: &str,
    evidence: &str,
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

pub fn run_overlay_remove(vault_path: &str, overlay_id: &str) -> AppResult<()> {
    let vault = vault_open(Path::new(vault_path))?;
    let conn = open_db(&Path::new(vault_path).join(vault.db.relative_path))?;
    lineage_overlay_remove(&conn, overlay_id)?;
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

#[cfg(test)]
mod tests {
    use super::{run_overlay_add, run_overlay_list, run_overlay_remove};
    use kc_core::db::open_db;
    use kc_core::ingest::ingest_bytes;
    use kc_core::lineage::lineage_overlay_list;
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
        drop(conn);

        let doc_id = ingested.doc_id.0.clone();
        let doc_node = format!("doc:{}", doc_id);
        let chunk_node = "chunk:cli-overlay";

        run_overlay_add(
            root.to_string_lossy().as_ref(),
            &doc_id,
            &doc_node,
            chunk_node,
            "related_to",
            "cli",
            "cli",
            2,
        )
        .expect("overlay add");
        run_overlay_list(root.to_string_lossy().as_ref(), &doc_id).expect("overlay list");

        let conn = open_db(&root.join("db/knowledge.sqlite")).expect("open db 2");
        let listed = lineage_overlay_list(&conn, &doc_id).expect("overlay list direct");
        assert_eq!(listed.len(), 1);
        let overlay_id = listed[0].overlay_id.clone();
        drop(conn);

        run_overlay_remove(root.to_string_lossy().as_ref(), &overlay_id).expect("overlay remove");
    }
}
