use kc_core::vault::{vault_init, vault_open};

#[test]
fn vault_init_creates_structure_and_vault_json() {
    let temp = tempfile::tempdir().expect("tempdir");
    let vault_root = temp.path().join("vault");

    let created = vault_init(&vault_root, "demo", 1234).expect("vault_init");
    assert_eq!(created.schema_version, 3);
    assert!(!created.encryption.enabled);
    assert!(!created.db_encryption.enabled);

    assert!(vault_root.join("vault.json").exists());
    assert!(vault_root.join("db").exists());
    assert!(vault_root.join("store/objects").exists());
    assert!(vault_root.join("Inbox/processed").exists());
    assert!(vault_root.join("index/vectors").exists());

    let opened = vault_open(&vault_root).expect("vault_open");
    assert_eq!(opened.schema_version, 3);
    assert_eq!(opened.vault_slug, "demo");
    assert!(!opened.encryption.enabled);
    assert!(!opened.db_encryption.enabled);
}

#[test]
fn vault_open_rejects_unsupported_schema_version() {
    let temp = tempfile::tempdir().expect("tempdir");
    let vault_root = temp.path().join("vault");
    std::fs::create_dir_all(&vault_root).expect("mkdir");

    let bad = serde_json::json!({
        "schema_version": 99,
        "vault_id": "2f9709fe-dda6-41d6-93c6-f1a0d5f9f3fd",
        "vault_slug": "bad",
        "created_at_ms": 1,
        "db": {"relative_path": "db/knowledge.sqlite"},
        "defaults": {
            "chunking_config_id": "chunking/default-v1",
            "embedding_model_id": "embedding/default-v1",
            "recency": {"enabled": false}
        },
        "toolchain": {
            "pdfium": {"identity": "pdfium:unconfigured"},
            "tesseract": {"identity": "tesseract:unconfigured"}
        }
    });

    std::fs::write(
        vault_root.join("vault.json"),
        serde_json::to_vec_pretty(&bad).expect("serialize"),
    )
    .expect("write");

    let err = vault_open(&vault_root).expect_err("should reject unsupported version");
    assert_eq!(err.code, "KC_VAULT_JSON_UNSUPPORTED_VERSION");
}

#[test]
fn vault_open_normalizes_legacy_v1_to_v3_defaults() {
    let temp = tempfile::tempdir().expect("tempdir");
    let vault_root = temp.path().join("vault");
    std::fs::create_dir_all(&vault_root).expect("mkdir");

    let legacy = serde_json::json!({
        "schema_version": 1,
        "vault_id": "2f9709fe-dda6-41d6-93c6-f1a0d5f9f3fd",
        "vault_slug": "legacy",
        "created_at_ms": 1,
        "db": {"relative_path": "db/knowledge.sqlite"},
        "defaults": {
            "chunking_config_id": "chunking/default-v1",
            "embedding_model_id": "embedding/default-v1",
            "recency": {"enabled": false}
        },
        "toolchain": {
            "pdfium": {"identity": "pdfium:unconfigured"},
            "tesseract": {"identity": "tesseract:unconfigured"}
        }
    });

    std::fs::write(
        vault_root.join("vault.json"),
        serde_json::to_vec_pretty(&legacy).expect("serialize"),
    )
    .expect("write");

    let opened = vault_open(&vault_root).expect("vault open legacy");
    assert_eq!(opened.schema_version, 3);
    assert_eq!(opened.vault_slug, "legacy");
    assert!(!opened.encryption.enabled);
    assert_eq!(opened.encryption.mode, "object_store_xchacha20poly1305");
    assert!(!opened.db_encryption.enabled);
}
