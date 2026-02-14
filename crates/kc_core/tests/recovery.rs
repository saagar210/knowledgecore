use kc_core::canon_json::to_canonical_bytes;
use kc_core::recovery::{generate_recovery_bundle, verify_recovery_bundle};
use kc_core::vault::vault_init;

#[test]
fn recovery_generate_and_verify_round_trip() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let vault = vault_init(&root.join("vault"), "demo", 1).expect("vault init");
    let output = root.join("recovery-out");

    let generated = generate_recovery_bundle(&vault.vault_id, &output, "vault-passphrase", 100)
        .expect("generate recovery");
    assert!(generated.bundle_path.exists());
    assert_eq!(generated.manifest.schema_version, 1);

    let verified = verify_recovery_bundle(
        &vault.vault_id,
        &generated.bundle_path,
        &generated.recovery_phrase,
    )
    .expect("verify recovery");
    assert_eq!(verified, generated.manifest);
}

#[test]
fn recovery_manifest_is_canonical_json() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let vault = vault_init(&root.join("vault"), "demo", 1).expect("vault init");
    let output = root.join("recovery-out");
    let generated = generate_recovery_bundle(&vault.vault_id, &output, "vault-passphrase", 100)
        .expect("generate recovery");
    let manifest_path = generated.bundle_path.join("recovery_manifest.json");
    let manifest_bytes = std::fs::read(&manifest_path).expect("read manifest bytes");
    let expected =
        to_canonical_bytes(&serde_json::to_value(&generated.manifest).expect("manifest value"))
            .expect("canonical bytes");
    assert_eq!(manifest_bytes, expected);
}

#[test]
fn recovery_verify_rejects_wrong_phrase() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let vault = vault_init(&root.join("vault"), "demo", 1).expect("vault init");
    let output = root.join("recovery-out");

    let generated = generate_recovery_bundle(&vault.vault_id, &output, "vault-passphrase", 100)
        .expect("generate recovery");
    let err = verify_recovery_bundle(&vault.vault_id, &generated.bundle_path, "wrong-phrase")
        .expect_err("verify should fail");
    assert_eq!(err.code, "KC_RECOVERY_PHRASE_INVALID");
}

#[test]
fn recovery_verify_rejects_tampered_blob() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let vault = vault_init(&root.join("vault"), "demo", 1).expect("vault init");
    let output = root.join("recovery-out");

    let generated = generate_recovery_bundle(&vault.vault_id, &output, "vault-passphrase", 100)
        .expect("generate recovery");
    let blob_path = generated.bundle_path.join("key_blob.enc");
    let mut blob = std::fs::read(&blob_path).expect("read blob");
    blob.push(0x42);
    std::fs::write(&blob_path, blob).expect("tamper blob");

    let err = verify_recovery_bundle(
        &vault.vault_id,
        &generated.bundle_path,
        &generated.recovery_phrase,
    )
    .expect_err("verify should fail");
    assert_eq!(err.code, "KC_RECOVERY_BUNDLE_INVALID");
}
