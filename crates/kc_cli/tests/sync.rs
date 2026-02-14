use kc_core::db::open_db;
use kc_core::object_store::ObjectStore;
use kc_core::sync::{sync_pull, sync_push};
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
    store
        .put_bytes(&conn, b"sync payload", 1)
        .expect("put object");

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
    assert!(
        push.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&push.stderr)
    );

    let status = Command::new(bin)
        .args([
            "sync",
            "status",
            vault_root.to_string_lossy().as_ref(),
            sync_target.to_string_lossy().as_ref(),
        ])
        .output()
        .expect("run sync status");
    assert!(
        status.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&status.stderr)
    );
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
    store
        .put_bytes(&conn, b"sync payload", 1)
        .expect("put object");

    let bin = env!("CARGO_BIN_EXE_kc_cli");

    let push = Command::new(bin)
        .env(
            "KC_SYNC_S3_EMULATE_ROOT",
            emulated_s3.to_string_lossy().as_ref(),
        )
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
        .env(
            "KC_SYNC_S3_EMULATE_ROOT",
            emulated_s3.to_string_lossy().as_ref(),
        )
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
        .env(
            "KC_SYNC_S3_EMULATE_ROOT",
            emulated_s3.to_string_lossy().as_ref(),
        )
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

#[test]
fn cli_sync_merge_preview_and_conservative_pull_work() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let vault_a = root.join("vault_a");
    let vault_b = root.join("vault_b");
    let sync_target = root.join("sync-target");

    vault_init(&vault_a, "a", 1).expect("vault a init");
    vault_init(&vault_b, "b", 1).expect("vault b init");

    let conn_a = open_db(&vault_a.join("db/knowledge.sqlite")).expect("open db a");
    let conn_b = open_db(&vault_b.join("db/knowledge.sqlite")).expect("open db b");
    let store_a = ObjectStore::new(vault_a.join("store/objects"));
    let store_b = ObjectStore::new(vault_b.join("store/objects"));

    store_a
        .put_bytes(&conn_a, b"baseline", 1)
        .expect("put baseline");
    sync_push(&conn_a, &vault_a, &sync_target, 100).expect("push baseline");
    sync_pull(&conn_b, &vault_b, &sync_target, 101).expect("pull baseline");
    let conn_b = open_db(&vault_b.join("db/knowledge.sqlite")).expect("reopen db b");

    store_a
        .put_bytes(&conn_a, b"local-change", 2)
        .expect("put local change");
    store_b
        .put_bytes(&conn_b, b"remote-change", 2)
        .expect("put remote change");
    sync_push(&conn_b, &vault_b, &sync_target, 200).expect("push remote change");

    let bin = env!("CARGO_BIN_EXE_kc_cli");
    let merge_preview = Command::new(bin)
        .args([
            "sync",
            "merge-preview",
            vault_a.to_string_lossy().as_ref(),
            sync_target.to_string_lossy().as_ref(),
            "--now-ms",
            "300",
        ])
        .output()
        .expect("run merge preview");
    assert!(
        merge_preview.status.success(),
        "merge preview stderr: {}",
        String::from_utf8_lossy(&merge_preview.stderr)
    );
    let preview_stdout = String::from_utf8(merge_preview.stdout).expect("preview utf8");
    assert!(preview_stdout.contains("\"safe\": true"));

    let conflict_pull = Command::new(bin)
        .args([
            "sync",
            "pull",
            vault_a.to_string_lossy().as_ref(),
            sync_target.to_string_lossy().as_ref(),
            "--now-ms",
            "301",
        ])
        .output()
        .expect("run pull without auto merge");
    assert!(
        !conflict_pull.status.success(),
        "expected conflict pull to fail"
    );

    let merged_pull = Command::new(bin)
        .args([
            "sync",
            "pull",
            vault_a.to_string_lossy().as_ref(),
            sync_target.to_string_lossy().as_ref(),
            "--auto-merge",
            "conservative",
            "--now-ms",
            "302",
        ])
        .output()
        .expect("run pull with conservative auto merge");
    assert!(
        merged_pull.status.success(),
        "merged pull stderr: {}",
        String::from_utf8_lossy(&merged_pull.stderr)
    );
}
