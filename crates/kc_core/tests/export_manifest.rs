use kc_core::chunking::{default_chunking_config_v1, hash_chunking_config};
use kc_core::db::open_db;
use kc_core::export::{export_bundle, ExportOptions};
use kc_core::hashing::blake3_hex_prefixed;
use kc_core::object_store::ObjectStore;
use kc_core::vault::vault_init;

#[test]
fn export_manifest_has_deterministic_object_order() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let vault_root = root.join("vault");
    let export_root = root.join("exports");

    vault_init(&vault_root, "demo", 1000).expect("vault init");
    let conn = open_db(&vault_root.join("db/knowledge.sqlite")).expect("open db");
    let store = ObjectStore::new(vault_root.join("store/objects"));

    store.put_bytes(&conn, b"bbb", 1).expect("store bbb");
    store.put_bytes(&conn, b"aaa", 2).expect("store aaa");

    let bundle = export_bundle(
        &vault_root,
        &export_root,
        &ExportOptions {
            include_vectors: false,
            as_zip: false,
        },
        123,
    )
    .expect("export");

    let manifest: serde_json::Value = serde_json::from_slice(
        &std::fs::read(bundle.join("manifest.json")).expect("read manifest"),
    )
    .expect("parse manifest");

    let objects = manifest
        .get("objects")
        .and_then(|v| v.as_array())
        .expect("objects array");

    for object in objects {
        assert!(object
            .get("storage_hash")
            .and_then(|v| v.as_str())
            .is_some());
        assert!(object.get("encrypted").and_then(|v| v.as_bool()).is_some());
    }

    let hashes: Vec<String> = objects
        .iter()
        .map(|o| {
            o.get("hash")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string()
        })
        .collect();

    let mut sorted = hashes.clone();
    sorted.sort();
    assert_eq!(hashes, sorted);

    let db_rel = manifest
        .get("db")
        .and_then(|v| v.get("relative_path"))
        .and_then(|v| v.as_str())
        .expect("db relative path");
    let db_hash = manifest
        .get("db")
        .and_then(|v| v.get("hash"))
        .and_then(|v| v.as_str())
        .expect("db hash");

    let db_actual = blake3_hex_prefixed(&std::fs::read(bundle.join(db_rel)).expect("read db"));
    assert_eq!(db_hash, db_actual);

    let chunking_hash = manifest
        .get("chunking_config_hash")
        .and_then(|v| v.as_str())
        .expect("chunking hash");
    let expected_chunking_hash =
        hash_chunking_config(&default_chunking_config_v1()).expect("hash default chunking config");
    assert_eq!(chunking_hash, expected_chunking_hash.0);

    let encryption = manifest.get("encryption").expect("encryption block");
    assert_eq!(
        encryption.get("enabled").and_then(|v| v.as_bool()),
        Some(false)
    );
    let db_encryption = manifest.get("db_encryption").expect("db_encryption block");
    assert_eq!(
        db_encryption.get("enabled").and_then(|v| v.as_bool()),
        Some(false)
    );
    assert_eq!(
        db_encryption.get("mode").and_then(|v| v.as_str()),
        Some("sqlcipher_v4")
    );
    let recovery_escrow = manifest
        .get("recovery_escrow")
        .expect("recovery_escrow block");
    assert_eq!(
        recovery_escrow.get("enabled").and_then(|v| v.as_bool()),
        Some(false)
    );
    assert_eq!(
        recovery_escrow.get("provider").and_then(|v| v.as_str()),
        Some("none")
    );
    assert_eq!(
        recovery_escrow
            .get("providers")
            .and_then(|v| v.as_array())
            .map(|v| v.len()),
        Some(0)
    );
    assert!(recovery_escrow
        .get("updated_at_ms")
        .expect("updated_at_ms")
        .is_null());
    assert!(recovery_escrow
        .get("descriptor")
        .expect("descriptor")
        .is_null());
    assert_eq!(
        recovery_escrow
            .get("escrow_descriptors")
            .and_then(|v| v.as_array())
            .map(|v| v.len()),
        Some(0)
    );
    assert_eq!(
        manifest
            .get("packaging")
            .and_then(|v| v.get("format"))
            .and_then(|v| v.as_str()),
        Some("folder")
    );
    assert_eq!(
        manifest
            .get("schema_versions")
            .and_then(|v| v.get("vault"))
            .and_then(|v| v.as_i64()),
        Some(3)
    );
}

#[test]
fn export_manifest_includes_vectors_when_enabled() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let vault_root = root.join("vault");
    let export_root = root.join("exports");

    vault_init(&vault_root, "demo", 1000).expect("vault init");
    let _conn = open_db(&vault_root.join("db/knowledge.sqlite")).expect("open db");
    std::fs::create_dir_all(vault_root.join("index/vectors")).expect("mkdir vectors");
    std::fs::write(vault_root.join("index/vectors/b.vec"), b"bbb").expect("write b");
    std::fs::write(vault_root.join("index/vectors/a.vec"), b"aaa").expect("write a");

    let bundle = export_bundle(
        &vault_root,
        &export_root,
        &ExportOptions {
            include_vectors: true,
            as_zip: false,
        },
        124,
    )
    .expect("export");

    let manifest: serde_json::Value = serde_json::from_slice(
        &std::fs::read(bundle.join("manifest.json")).expect("read manifest"),
    )
    .expect("parse manifest");

    let vectors = manifest
        .get("indexes")
        .and_then(|x| x.get("vectors"))
        .and_then(|x| x.as_array())
        .expect("vectors array");
    assert_eq!(vectors.len(), 2);

    let rels: Vec<String> = vectors
        .iter()
        .map(|v| {
            v.get("relative_path")
                .and_then(|x| x.as_str())
                .unwrap_or_default()
                .to_string()
        })
        .collect();
    assert_eq!(
        rels,
        vec![
            "index/vectors/a.vec".to_string(),
            "index/vectors/b.vec".to_string()
        ]
    );
}

#[test]
fn export_manifest_includes_recovery_escrow_descriptor_when_enabled() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let vault_root = root.join("vault");
    let export_root = root.join("exports");

    vault_init(&vault_root, "demo", 1000).expect("vault init");
    let conn = open_db(&vault_root.join("db/knowledge.sqlite")).expect("open db");
    conn.execute(
        "INSERT INTO recovery_escrow_configs (provider_id, enabled, descriptor_json, updated_at_ms)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![
            "aws",
            1,
            r#"{"provider":"aws","provider_ref":"secret://vault/demo","key_id":"kms://demo","wrapped_at_ms":2000}"#,
            2000i64
        ],
    )
    .expect("insert escrow config");
    conn.execute(
        "INSERT INTO recovery_escrow_configs (provider_id, enabled, descriptor_json, updated_at_ms)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![
            "gcp",
            1,
            r#"{"provider":"gcp","provider_ref":"secret://vault/gcp","key_id":"gcp-kms://demo","wrapped_at_ms":2100}"#,
            2100i64
        ],
    )
    .expect("insert escrow config gcp");

    let bundle = export_bundle(
        &vault_root,
        &export_root,
        &ExportOptions {
            include_vectors: false,
            as_zip: false,
        },
        125,
    )
    .expect("export");

    let manifest: serde_json::Value = serde_json::from_slice(
        &std::fs::read(bundle.join("manifest.json")).expect("read manifest"),
    )
    .expect("parse manifest");
    let recovery_escrow = manifest
        .get("recovery_escrow")
        .expect("recovery_escrow block");
    assert_eq!(
        recovery_escrow.get("enabled").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        recovery_escrow.get("provider").and_then(|v| v.as_str()),
        Some("multi")
    );
    assert_eq!(
        recovery_escrow
            .get("updated_at_ms")
            .and_then(|v| v.as_i64()),
        Some(2100)
    );
    assert_eq!(
        recovery_escrow
            .get("providers")
            .and_then(|v| v.as_array())
            .map(|items| items
                .iter()
                .filter_map(|x| x.as_str())
                .map(str::to_string)
                .collect::<Vec<_>>()),
        Some(vec!["aws".to_string(), "gcp".to_string()])
    );
    assert_eq!(
        recovery_escrow
            .get("descriptor")
            .and_then(|v| v.get("provider"))
            .and_then(|v| v.as_str()),
        Some("aws")
    );
    let descriptors = recovery_escrow
        .get("escrow_descriptors")
        .and_then(|v| v.as_array())
        .expect("escrow_descriptors array");
    assert_eq!(descriptors.len(), 2);
    assert_eq!(
        descriptors[0].get("provider").and_then(|v| v.as_str()),
        Some("aws")
    );
    assert_eq!(
        descriptors[1].get("provider").and_then(|v| v.as_str()),
        Some("gcp")
    );
}

#[test]
fn export_manifest_orders_expanded_recovery_escrow_providers() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let vault_root = root.join("vault");
    let export_root = root.join("exports");

    vault_init(&vault_root, "demo", 1000).expect("vault init");
    let conn = open_db(&vault_root.join("db/knowledge.sqlite")).expect("open db");

    let rows = [
        (
            "private_kms",
            r#"{"provider":"private_kms","provider_ref":"secret://vault/private","key_id":"private-kms://demo","wrapped_at_ms":2600}"#,
            2600i64,
        ),
        (
            "local",
            r#"{"provider":"local","provider_ref":"secret://vault/local","key_id":"local://demo","wrapped_at_ms":2500}"#,
            2500i64,
        ),
        (
            "hsm",
            r#"{"provider":"hsm","provider_ref":"secret://vault/hsm","key_id":"hsm://demo","wrapped_at_ms":2400}"#,
            2400i64,
        ),
        (
            "azure",
            r#"{"provider":"azure","provider_ref":"secret://vault/azure","key_id":"azure-kv://demo","wrapped_at_ms":2300}"#,
            2300i64,
        ),
        (
            "gcp",
            r#"{"provider":"gcp","provider_ref":"secret://vault/gcp","key_id":"gcp-kms://demo","wrapped_at_ms":2200}"#,
            2200i64,
        ),
        (
            "aws",
            r#"{"provider":"aws","provider_ref":"secret://vault/aws","key_id":"kms://demo","wrapped_at_ms":2100}"#,
            2100i64,
        ),
    ];
    for (provider_id, descriptor_json, updated_at_ms) in rows {
        conn.execute(
            "INSERT INTO recovery_escrow_configs (provider_id, enabled, descriptor_json, updated_at_ms)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![provider_id, 1, descriptor_json, updated_at_ms],
        )
        .expect("insert escrow config");
    }

    let bundle = export_bundle(
        &vault_root,
        &export_root,
        &ExportOptions {
            include_vectors: false,
            as_zip: false,
        },
        126,
    )
    .expect("export");

    let manifest: serde_json::Value = serde_json::from_slice(
        &std::fs::read(bundle.join("manifest.json")).expect("read manifest"),
    )
    .expect("parse manifest");
    let recovery_escrow = manifest
        .get("recovery_escrow")
        .expect("recovery_escrow block");
    assert_eq!(
        recovery_escrow.get("provider").and_then(|v| v.as_str()),
        Some("multi")
    );
    assert_eq!(
        recovery_escrow
            .get("updated_at_ms")
            .and_then(|v| v.as_i64()),
        Some(2600)
    );
    assert_eq!(
        recovery_escrow
            .get("providers")
            .and_then(|v| v.as_array())
            .map(|items| items
                .iter()
                .filter_map(|x| x.as_str())
                .map(str::to_string)
                .collect::<Vec<_>>()),
        Some(vec![
            "aws".to_string(),
            "gcp".to_string(),
            "azure".to_string(),
            "hsm".to_string(),
            "local".to_string(),
            "private_kms".to_string(),
        ])
    );
    assert_eq!(
        recovery_escrow
            .get("descriptor")
            .and_then(|v| v.get("provider"))
            .and_then(|v| v.as_str()),
        Some("aws")
    );

    let descriptor_providers: Vec<String> = recovery_escrow
        .get("escrow_descriptors")
        .and_then(|v| v.as_array())
        .expect("escrow_descriptors array")
        .iter()
        .filter_map(|entry| entry.get("provider"))
        .filter_map(|value| value.as_str())
        .map(str::to_string)
        .collect();
    assert_eq!(
        descriptor_providers,
        vec![
            "aws".to_string(),
            "gcp".to_string(),
            "azure".to_string(),
            "hsm".to_string(),
            "local".to_string(),
            "private_kms".to_string(),
        ]
    );
}

#[test]
fn export_zip_is_byte_stable_for_identical_state() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let vault_root = root.join("vault");
    let export_root = root.join("exports");

    vault_init(&vault_root, "demo", 1000).expect("vault init");
    let conn = open_db(&vault_root.join("db/knowledge.sqlite")).expect("open db");
    let store = ObjectStore::new(vault_root.join("store/objects"));
    store
        .put_bytes(&conn, b"zip deterministic", 1)
        .expect("store object");

    let zip_a = export_bundle(
        &vault_root,
        &export_root,
        &ExportOptions {
            include_vectors: false,
            as_zip: true,
        },
        300,
    )
    .expect("export zip a");
    let zip_b = export_bundle(
        &vault_root,
        &export_root,
        &ExportOptions {
            include_vectors: false,
            as_zip: true,
        },
        300,
    )
    .expect("export zip b");

    assert_eq!(zip_a, zip_b);
    let bytes_a = std::fs::read(&zip_a).expect("read zip a");
    let bytes_b = std::fs::read(&zip_b).expect("read zip b");
    assert_eq!(bytes_a, bytes_b);
}
