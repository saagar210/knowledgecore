use kc_core::db::open_db;
use kc_core::object_store::ObjectStore;
use kc_core::vault::vault_init;
use std::process::Command;

#[test]
fn cli_sync_push_and_status_work() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let vault_root = root.join("vault");
    let sync_target = root.join("sync-target");

    vault_init(&vault_root, "demo", 1).expect("vault init");
    let conn = open_db(&vault_root.join("db/knowledge.sqlite")).expect("open db");
    let store = ObjectStore::new(vault_root.join("store/objects"));
    store.put_bytes(&conn, b"sync payload", 1).expect("put object");

    let bin = env!("CARGO_BIN_EXE_kc_cli");

    let push = Command::new(bin)
        .args([
            "sync",
            "push",
            vault_root.to_string_lossy().as_ref(),
            sync_target.to_string_lossy().as_ref(),
            "--now-ms",
            "100",
        ])
        .output()
        .expect("run sync push");
    assert!(push.status.success(), "stderr: {}", String::from_utf8_lossy(&push.stderr));

    let status = Command::new(bin)
        .args([
            "sync",
            "status",
            vault_root.to_string_lossy().as_ref(),
            sync_target.to_string_lossy().as_ref(),
        ])
        .output()
        .expect("run sync status");
    assert!(status.status.success(), "stderr: {}", String::from_utf8_lossy(&status.stderr));
    let stdout = String::from_utf8(status.stdout).expect("utf8 status");
    assert!(stdout.contains("\"remote_head\""));
}
