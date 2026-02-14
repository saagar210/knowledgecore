use kc_core::db::{open_db, schema_version};
use kc_core::vault::{vault_init, vault_paths, vault_save};
use std::sync::{Mutex, OnceLock};

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[test]
fn db_encryption_requires_passphrase_when_enabled() {
    let _guard = env_lock().lock().expect("env lock");
    std::env::remove_var("KC_VAULT_DB_PASSPHRASE");
    std::env::remove_var("KC_VAULT_PASSPHRASE");

    let root = tempfile::tempdir().expect("tempdir").keep();
    let mut vault = vault_init(&root, "demo", 1).expect("vault init");
    vault.db_encryption.enabled = true;
    vault.db_encryption.key_reference = Some(format!("vaultdb:{}", vault.vault_id));
    vault_save(&root, &vault).expect("vault save");

    let err = open_db(&vault_paths(&root).db).expect_err("expected locked db error");
    assert_eq!(err.code, "KC_DB_LOCKED");
}

#[test]
fn db_encryption_key_validation_is_deterministic() {
    let _guard = env_lock().lock().expect("env lock");
    std::env::remove_var("KC_VAULT_DB_PASSPHRASE");
    std::env::remove_var("KC_VAULT_PASSPHRASE");

    let root = tempfile::tempdir().expect("tempdir").keep();
    let mut vault = vault_init(&root, "demo", 1).expect("vault init");
    vault.db_encryption.enabled = true;
    vault.db_encryption.key_reference = Some(format!("vaultdb:{}", vault.vault_id));
    vault_save(&root, &vault).expect("vault save");

    std::env::set_var("KC_VAULT_DB_PASSPHRASE", "correct-passphrase");
    let conn = open_db(&vault_paths(&root).db).expect("open encrypted db with passphrase");
    assert_eq!(schema_version(&conn).expect("schema version"), 2);
    drop(conn);

    std::env::set_var("KC_VAULT_DB_PASSPHRASE", "wrong-passphrase");
    let err = open_db(&vault_paths(&root).db).expect_err("expected invalid key error");
    assert_eq!(err.code, "KC_DB_KEY_INVALID");

    std::env::remove_var("KC_VAULT_DB_PASSPHRASE");
    std::env::remove_var("KC_VAULT_PASSPHRASE");
}
