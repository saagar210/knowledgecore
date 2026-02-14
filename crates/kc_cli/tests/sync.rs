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

#[test]
fn cli_sync_supports_s3_uri_with_emulation() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let pull_root = tempfile::tempdir().expect("pull tempdir").keep();
    let vault_root = root.join("vault");
    let pull_vault_root = pull_root.join("vault");
    let emulated_s3 = root.join("emulated-s3");
    let target_uri = "s3://demo-bucket/kc";

    vault_init(&vault_root, "demo", 1).expect("vault init");
    vault_init(&pull_vault_root, "pull-demo", 1).expect("pull vault init");

    let conn = open_db(&vault_root.join("db/knowledge.sqlite")).expect("open db");
    let store = ObjectStore::new(vault_root.join("store/objects"));
    store.put_bytes(&conn, b"sync payload", 1).expect("put object");

    let bin = env!("CARGO_BIN_EXE_kc_cli");

    let push = Command::new(bin)
        .env("KC_SYNC_S3_EMULATE_ROOT", emulated_s3.to_string_lossy().as_ref())
        .env("KC_VAULT_PASSPHRASE", "cli-sync-passphrase")
        .args([
            "sync",
            "push",
            vault_root.to_string_lossy().as_ref(),
            target_uri,
            "--now-ms",
            "100",
        ])
        .output()
        .expect("run sync push");
    assert!(
        push.status.success(),
        "push stderr: {}",
        String::from_utf8_lossy(&push.stderr)
    );

    let pull = Command::new(bin)
        .env("KC_SYNC_S3_EMULATE_ROOT", emulated_s3.to_string_lossy().as_ref())
        .env("KC_VAULT_PASSPHRASE", "cli-sync-passphrase")
        .args([
            "sync",
            "pull",
            pull_vault_root.to_string_lossy().as_ref(),
            target_uri,
            "--now-ms",
            "101",
        ])
        .output()
        .expect("run sync pull");
    assert!(
        pull.status.success(),
        "pull stderr: {}",
        String::from_utf8_lossy(&pull.stderr)
    );

    let status = Command::new(bin)
        .env("KC_SYNC_S3_EMULATE_ROOT", emulated_s3.to_string_lossy().as_ref())
        .args([
            "sync",
            "status",
            vault_root.to_string_lossy().as_ref(),
            target_uri,
        ])
        .output()
        .expect("run sync status");
    assert!(
        status.status.success(),
        "status stderr: {}",
        String::from_utf8_lossy(&status.stderr)
    );
    let stdout = String::from_utf8(status.stdout).expect("utf8 status");
    assert!(stdout.contains(target_uri));
}
