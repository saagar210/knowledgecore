use kc_core::db::{db_is_unlocked, db_lock, db_unlock, migrate_db_to_sqlcipher, open_db, schema_version};
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

#[test]
fn db_unlock_session_allows_open_without_env() {
    let _guard = env_lock().lock().expect("env lock");
    std::env::remove_var("KC_VAULT_DB_PASSPHRASE");
    std::env::remove_var("KC_VAULT_PASSPHRASE");

    let root = tempfile::tempdir().expect("tempdir").keep();
    let mut vault = vault_init(&root, "demo", 1).expect("vault init");
    vault.db_encryption.enabled = true;
    vault.db_encryption.key_reference = Some(format!("vaultdb:{}", vault.vault_id));
    vault_save(&root, &vault).expect("vault save");
    let db_path = vault_paths(&root).db;

    let locked = open_db(&db_path).expect_err("expected db locked");
    assert_eq!(locked.code, "KC_DB_LOCKED");

    db_unlock(&root, &db_path, "correct-passphrase").expect("db unlock");
    assert!(db_is_unlocked(&root));
    let conn = open_db(&db_path).expect("open db with unlock session");
    assert_eq!(schema_version(&conn).expect("schema version"), 2);
    drop(conn);

    db_lock(&root).expect("db lock");
    assert!(!db_is_unlocked(&root));
}

#[test]
fn db_migration_to_sqlcipher_requires_valid_key_after_migrate() {
    let _guard = env_lock().lock().expect("env lock");
    std::env::remove_var("KC_VAULT_DB_PASSPHRASE");
    std::env::remove_var("KC_VAULT_PASSPHRASE");

    let root = tempfile::tempdir().expect("tempdir").keep();
    let mut vault = vault_init(&root, "demo", 1).expect("vault init");
    let db_path = vault_paths(&root).db;

    let outcome = migrate_db_to_sqlcipher(&db_path, "migration-passphrase").expect("migrate db");
    assert_eq!(outcome, kc_core::db::DbMigrationOutcome::Migrated);

    vault.db_encryption.enabled = true;
    vault.db_encryption.key_reference = Some(format!("vaultdb:{}", vault.vault_id));
    vault_save(&root, &vault).expect("vault save");

    let locked = open_db(&db_path).expect_err("expected locked db");
    assert_eq!(locked.code, "KC_DB_LOCKED");

    std::env::set_var("KC_VAULT_DB_PASSPHRASE", "wrong-passphrase");
    let wrong = open_db(&db_path).expect_err("expected invalid key");
    assert_eq!(wrong.code, "KC_DB_KEY_INVALID");

    std::env::set_var("KC_VAULT_DB_PASSPHRASE", "migration-passphrase");
    let conn = open_db(&db_path).expect("open migrated encrypted db");
    assert_eq!(schema_version(&conn).expect("schema version"), 2);

    std::env::remove_var("KC_VAULT_DB_PASSPHRASE");
    std::env::remove_var("KC_VAULT_PASSPHRASE");
}
