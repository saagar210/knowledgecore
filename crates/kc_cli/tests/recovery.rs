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
