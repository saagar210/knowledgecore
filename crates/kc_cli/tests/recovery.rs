use kc_core::vault::vault_init;
use std::process::Command;

#[test]
fn cli_recovery_generate_and_verify_round_trip() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let vault_root = root.join("vault");
    let output_root = root.join("recovery");
    vault_init(&vault_root, "demo", 1).expect("vault init");

    let bin = env!("CARGO_BIN_EXE_kc_cli");
    let generate = Command::new(bin)
        .env("KC_VAULT_PASSPHRASE", "demo-passphrase")
        .args([
            "vault",
            "recovery",
            "generate",
            vault_root.to_string_lossy().as_ref(),
            "--output",
            output_root.to_string_lossy().as_ref(),
            "--passphrase-env",
            "KC_VAULT_PASSPHRASE",
            "--now-ms",
            "100",
        ])
        .output()
        .expect("run recovery generate");
    assert!(
        generate.status.success(),
        "generate stderr: {}",
        String::from_utf8_lossy(&generate.stderr)
    );

    let generated_json: serde_json::Value =
        serde_json::from_slice(&generate.stdout).expect("parse generate stdout");
    let bundle_path = generated_json
        .get("bundle_path")
        .and_then(|v| v.as_str())
        .expect("bundle_path");
    let phrase = generated_json
        .get("recovery_phrase")
        .and_then(|v| v.as_str())
        .expect("recovery_phrase");

    let verify = Command::new(bin)
        .env("KC_RECOVERY_PHRASE", phrase)
        .args([
            "vault",
            "recovery",
            "verify",
            vault_root.to_string_lossy().as_ref(),
            "--bundle",
            bundle_path,
            "--phrase-env",
            "KC_RECOVERY_PHRASE",
        ])
        .output()
        .expect("run recovery verify");
    assert!(
        verify.status.success(),
        "verify stderr: {}",
        String::from_utf8_lossy(&verify.stderr)
    );
}

#[test]
fn cli_recovery_verify_fails_on_phrase_mismatch() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let vault_root = root.join("vault");
    let output_root = root.join("recovery");
    vault_init(&vault_root, "demo", 1).expect("vault init");

    let bin = env!("CARGO_BIN_EXE_kc_cli");
    let generate = Command::new(bin)
        .env("KC_VAULT_PASSPHRASE", "demo-passphrase")
        .args([
            "vault",
            "recovery",
            "generate",
            vault_root.to_string_lossy().as_ref(),
            "--output",
            output_root.to_string_lossy().as_ref(),
            "--passphrase-env",
            "KC_VAULT_PASSPHRASE",
            "--now-ms",
            "100",
        ])
        .output()
        .expect("run recovery generate");
    assert!(
        generate.status.success(),
        "generate stderr: {}",
        String::from_utf8_lossy(&generate.stderr)
    );
    let generated_json: serde_json::Value =
        serde_json::from_slice(&generate.stdout).expect("parse generate stdout");
    let bundle_path = generated_json
        .get("bundle_path")
        .and_then(|v| v.as_str())
        .expect("bundle_path");

    let verify = Command::new(bin)
        .env("KC_RECOVERY_PHRASE", "wrong-phrase")
        .args([
            "vault",
            "recovery",
            "verify",
            vault_root.to_string_lossy().as_ref(),
            "--bundle",
            bundle_path,
            "--phrase-env",
            "KC_RECOVERY_PHRASE",
        ])
        .output()
        .expect("run recovery verify");
    assert!(
        !verify.status.success(),
        "verify unexpectedly succeeded: {}",
        String::from_utf8_lossy(&verify.stdout)
    );
    assert!(String::from_utf8_lossy(&verify.stderr).contains("KC_RECOVERY_PHRASE_INVALID"));
}

#[test]
fn cli_recovery_escrow_enable_rotate_restore_round_trip() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let vault_root = root.join("vault");
    let emu_root = root.join("escrow-emulation");
    std::fs::create_dir_all(&emu_root).expect("create emu dir");
    vault_init(&vault_root, "demo", 1).expect("vault init");

    let bin = env!("CARGO_BIN_EXE_kc_cli");
    let enable = Command::new(bin)
        .env(
            "KC_RECOVERY_ESCROW_AWS_EMULATE_DIR",
            emu_root.to_string_lossy().to_string(),
        )
        .env("KC_RECOVERY_ESCROW_AWS_KMS_KEY_ID", "alias/kc-test")
        .args([
            "vault",
            "recovery",
            "escrow",
            "enable",
            vault_root.to_string_lossy().as_ref(),
            "--provider",
            "aws",
            "--now-ms",
            "10",
        ])
        .output()
        .expect("run escrow enable");
    assert!(
        enable.status.success(),
        "enable stderr: {}",
        String::from_utf8_lossy(&enable.stderr)
    );

    let rotate = Command::new(bin)
        .env(
            "KC_RECOVERY_ESCROW_AWS_EMULATE_DIR",
            emu_root.to_string_lossy().to_string(),
        )
        .env("KC_RECOVERY_ESCROW_AWS_KMS_KEY_ID", "alias/kc-test")
        .env("KC_VAULT_PASSPHRASE", "demo-passphrase")
        .args([
            "vault",
            "recovery",
            "escrow",
            "rotate",
            vault_root.to_string_lossy().as_ref(),
            "--passphrase-env",
            "KC_VAULT_PASSPHRASE",
            "--now-ms",
            "11",
        ])
        .output()
        .expect("run escrow rotate");
    assert!(
        rotate.status.success(),
        "rotate stderr: {}",
        String::from_utf8_lossy(&rotate.stderr)
    );
    let rotate_json: serde_json::Value =
        serde_json::from_slice(&rotate.stdout).expect("parse rotate stdout");
    let bundle_path = rotate_json
        .get("bundle_path")
        .and_then(|v| v.as_str())
        .expect("bundle path");
    let bundle = std::path::PathBuf::from(bundle_path);
    let blob_path = bundle.join("key_blob.enc");
    std::fs::remove_file(&blob_path).expect("remove key blob");

    let restore = Command::new(bin)
        .env(
            "KC_RECOVERY_ESCROW_AWS_EMULATE_DIR",
            emu_root.to_string_lossy().to_string(),
        )
        .env("KC_RECOVERY_ESCROW_AWS_KMS_KEY_ID", "alias/kc-test")
        .args([
            "vault",
            "recovery",
            "escrow",
            "restore",
            vault_root.to_string_lossy().as_ref(),
            "--bundle",
            bundle_path,
            "--now-ms",
            "12",
        ])
        .output()
        .expect("run escrow restore");
    assert!(
        restore.status.success(),
        "restore stderr: {}",
        String::from_utf8_lossy(&restore.stderr)
    );
    assert!(blob_path.exists(), "restored blob should exist");
}
