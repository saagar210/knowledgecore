use kc_cli::verifier::{verify_bundle, verify_sync_head_payload};
use kc_core::hashing::blake3_hex_prefixed;

fn base_manifest(db_hash: String) -> serde_json::Value {
    serde_json::json!({
        "manifest_version": 1,
        "vault_id": "123e4567-e89b-12d3-a456-426614174000",
        "schema_versions": {
            "vault": 3,
            "locator": 1,
            "app_error": 1,
            "rpc": 1
        },
        "encryption": {
            "enabled": false,
            "mode": "object_store_xchacha20poly1305",
            "key_reference": serde_json::Value::Null,
            "kdf": {
                "algorithm": "argon2id",
                "memory_kib": 65536,
                "iterations": 3,
                "parallelism": 1,
                "salt_id": "vault-kdf-salt-v1"
            }
        },
        "db_encryption": {
            "enabled": false,
            "mode": "sqlcipher_v4",
            "key_reference": serde_json::Value::Null,
            "kdf": {
                "algorithm": "pbkdf2_hmac_sha512"
            }
        },
        "packaging": {
            "format": "folder",
            "zip_policy": {
                "compression": "stored",
                "mtime": "1980-01-01T00:00:00Z",
                "file_mode": "0644"
            }
        },
        "chunking_config_hash": "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "db": {
            "relative_path": "db/knowledge.sqlite",
            "hash": db_hash
        },
        "objects": [],
        "indexes": {}
    })
}

fn write_manifest(bundle: &std::path::Path, manifest: &serde_json::Value) {
    std::fs::write(
        bundle.join("manifest.json"),
        serde_json::to_vec(manifest).expect("manifest json"),
    )
    .expect("write manifest");
}

#[test]
fn verifier_ok_bundle() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let bundle = root.join("bundle");
    std::fs::create_dir_all(bundle.join("store/objects/aa")).expect("mkdir");

    let db_bytes = b"db";
    std::fs::create_dir_all(bundle.join("db")).expect("mkdir db");
    std::fs::write(bundle.join("db/knowledge.sqlite"), db_bytes).expect("write db");

    let obj_bytes = b"hello";
    let obj_hash = blake3_hex_prefixed(obj_bytes);
    std::fs::write(bundle.join(format!("store/objects/aa/{}", obj_hash)), obj_bytes).expect("write obj");

    let mut manifest = base_manifest(blake3_hex_prefixed(db_bytes));
    manifest["objects"] = serde_json::json!([
        {
            "relative_path": format!("store/objects/aa/{}", obj_hash),
            "hash": obj_hash,
            "storage_hash": blake3_hex_prefixed(obj_bytes),
            "encrypted": false,
            "bytes": obj_bytes.len()
        }
    ]);
    write_manifest(&bundle, &manifest);

    let (code, report) = verify_bundle(&bundle).expect("verify");
    assert_eq!(code, 0);
    assert_eq!(report.status, "ok");
}

#[test]
fn verifier_reports_db_mismatch_code() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let bundle = root.join("bundle2");
    std::fs::create_dir_all(bundle.join("db")).expect("mkdir db");
    std::fs::write(bundle.join("db/knowledge.sqlite"), b"db").expect("write db");

    let manifest = base_manifest(
        "blake3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
    );
    write_manifest(&bundle, &manifest);

    let (code, report) = verify_bundle(&bundle).expect("verify");
    assert_eq!(code, 31);
    assert_eq!(report.status, "failed");
}

#[test]
fn verifier_reports_schema_invalid_for_missing_required_fields() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let bundle = root.join("bundle3");
    std::fs::create_dir_all(&bundle).expect("mkdir");

    let manifest = serde_json::json!({
        "manifest_version": 1,
        "db": { "relative_path": "db/knowledge.sqlite", "hash": "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" },
        "objects": []
    });
    write_manifest(&bundle, &manifest);

    let (code, report) = verify_bundle(&bundle).expect("verify");
    assert_eq!(code, 21);
    assert_eq!(report.errors[0].code, "MANIFEST_SCHEMA_INVALID");
}

#[test]
fn verifier_errors_are_sorted_by_code_then_path() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let bundle = root.join("bundle4");
    std::fs::create_dir_all(bundle.join("db")).expect("mkdir db");
    std::fs::write(bundle.join("db/knowledge.sqlite"), b"db").expect("write db");

    let mut manifest = base_manifest(
        "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
    );
    manifest["objects"] = serde_json::json!([
        {
            "relative_path": "store/objects/aa/obj2",
            "hash": "blake3:2222222222222222222222222222222222222222222222222222222222222222",
            "storage_hash": "blake3:2222222222222222222222222222222222222222222222222222222222222222",
            "encrypted": false,
            "bytes": 10
        },
        {
            "relative_path": "store/objects/aa/obj1",
            "hash": "blake3:1111111111111111111111111111111111111111111111111111111111111111",
            "storage_hash": "blake3:1111111111111111111111111111111111111111111111111111111111111111",
            "encrypted": false,
            "bytes": 10
        }
    ]);
    write_manifest(&bundle, &manifest);

    let (_code, report) = verify_bundle(&bundle).expect("verify");
    let sorted = report.errors.clone();
    let mut expected = sorted.clone();
    expected.sort_by(|a, b| a.code.cmp(&b.code).then(a.path.cmp(&b.path)));
    assert_eq!(
        sorted.iter().map(|e| (&e.code, &e.path)).collect::<Vec<_>>(),
        expected.iter().map(|e| (&e.code, &e.path)).collect::<Vec<_>>()
    );
}

#[test]
fn verifier_checks_vector_index_files() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let bundle = root.join("bundle5");
    std::fs::create_dir_all(bundle.join("db")).expect("mkdir db");
    std::fs::create_dir_all(bundle.join("index/vectors")).expect("mkdir vectors");
    std::fs::write(bundle.join("db/knowledge.sqlite"), b"db").expect("write db");
    std::fs::write(bundle.join("index/vectors/a.vec"), b"aaa").expect("write vec");

    let mut manifest = base_manifest(blake3_hex_prefixed(b"db"));
    manifest["indexes"] = serde_json::json!({
        "vectors": [{
            "relative_path": "index/vectors/a.vec",
            "hash": blake3_hex_prefixed(b"aaa"),
            "bytes": 3
        }]
    });
    write_manifest(&bundle, &manifest);

    let (code, report) = verify_bundle(&bundle).expect("verify");
    assert_eq!(code, 0);
    assert_eq!(report.checked.indexes, 1);
}

#[test]
fn verifier_reports_manifest_invalid_json_code() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let bundle = root.join("bundle6");
    std::fs::create_dir_all(&bundle).expect("mkdir");
    std::fs::write(bundle.join("manifest.json"), b"{ not-json").expect("write manifest");

    let (code, report) = verify_bundle(&bundle).expect("verify");
    assert_eq!(code, 20);
    assert_eq!(report.errors[0].code, "MANIFEST_INVALID_JSON");
}

#[test]
fn verifier_reports_object_missing_code() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let bundle = root.join("bundle7");
    std::fs::create_dir_all(bundle.join("db")).expect("mkdir db");
    std::fs::write(bundle.join("db/knowledge.sqlite"), b"db").expect("write db");

    let mut manifest = base_manifest(blake3_hex_prefixed(b"db"));
    manifest["objects"] = serde_json::json!([
        {
            "relative_path": "store/objects/aa/missing",
            "hash": "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "storage_hash": "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "encrypted": false,
            "bytes": 1
        }
    ]);
    write_manifest(&bundle, &manifest);

    let (code, report) = verify_bundle(&bundle).expect("verify");
    assert_eq!(code, 40);
    assert!(report.errors.iter().any(|e| e.code == "OBJECT_MISSING"));
}

#[test]
fn verifier_reports_object_hash_mismatch_code() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let bundle = root.join("bundle8");
    std::fs::create_dir_all(bundle.join("db")).expect("mkdir db");
    std::fs::create_dir_all(bundle.join("store/objects/aa")).expect("mkdir objects");
    std::fs::write(bundle.join("db/knowledge.sqlite"), b"db").expect("write db");
    std::fs::write(bundle.join("store/objects/aa/x"), b"actual").expect("write obj");

    let mut manifest = base_manifest(blake3_hex_prefixed(b"db"));
    manifest["objects"] = serde_json::json!([
        {
            "relative_path": "store/objects/aa/x",
            "hash": "blake3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "storage_hash": "blake3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "encrypted": false,
            "bytes": 6
        }
    ]);
    write_manifest(&bundle, &manifest);

    let (code, report) = verify_bundle(&bundle).expect("verify");
    assert_eq!(code, 41);
    assert!(report.errors.iter().any(|e| e.code == "OBJECT_HASH_MISMATCH"));
}

#[test]
fn verifier_reports_internal_error_code_for_unreadable_object_path() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let bundle = root.join("bundle9");
    std::fs::create_dir_all(bundle.join("db")).expect("mkdir db");
    std::fs::create_dir_all(bundle.join("store/objects/aa/dir_as_object"))
        .expect("mkdir object dir");
    std::fs::write(bundle.join("db/knowledge.sqlite"), b"db").expect("write db");

    let mut manifest = base_manifest(blake3_hex_prefixed(b"db"));
    manifest["objects"] = serde_json::json!([
        {
            "relative_path": "store/objects/aa/dir_as_object",
            "hash": "blake3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "storage_hash": "blake3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "encrypted": false,
            "bytes": 1
        }
    ]);
    write_manifest(&bundle, &manifest);

    let (code, report) = verify_bundle(&bundle).expect("verify");
    assert_eq!(code, 60);
    assert!(report.errors.iter().any(|e| e.code == "INTERNAL_ERROR"));
}

#[test]
fn verifier_reports_encryption_mismatch_when_enabled_bundle_has_plain_object() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let bundle = root.join("bundle10");
    std::fs::create_dir_all(bundle.join("db")).expect("mkdir db");
    std::fs::create_dir_all(bundle.join("store/objects/aa")).expect("mkdir objects");
    std::fs::write(bundle.join("db/knowledge.sqlite"), b"db").expect("write db");

    let plaintext = b"plaintext-object";
    let obj_hash = blake3_hex_prefixed(plaintext);
    std::fs::write(bundle.join(format!("store/objects/aa/{}", obj_hash)), plaintext).expect("write obj");

    let mut manifest = base_manifest(blake3_hex_prefixed(b"db"));
    manifest["encryption"]["enabled"] = serde_json::json!(true);
    manifest["objects"] = serde_json::json!([
        {
            "relative_path": format!("store/objects/aa/{}", obj_hash),
            "hash": obj_hash,
            "storage_hash": blake3_hex_prefixed(plaintext),
            "encrypted": false,
            "bytes": plaintext.len()
        }
    ]);
    write_manifest(&bundle, &manifest);

    let (code, report) = verify_bundle(&bundle).expect("verify");
    assert_eq!(code, 41);
    assert!(
        report
            .errors
            .iter()
            .any(|e| e.code == "OBJECT_ENCRYPTION_MISMATCH")
    );
}

#[test]
fn verifier_reports_db_encryption_mismatch_when_plain_db_claims_encrypted() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let bundle = root.join("bundle_dbenc");
    std::fs::create_dir_all(bundle.join("db")).expect("mkdir db");
    let plaintext_sqlite_header = b"SQLite format 3\0fixture";
    std::fs::write(bundle.join("db/knowledge.sqlite"), plaintext_sqlite_header).expect("write db");

    let mut manifest = base_manifest(blake3_hex_prefixed(plaintext_sqlite_header));
    manifest["db_encryption"]["enabled"] = serde_json::json!(true);
    write_manifest(&bundle, &manifest);

    let (code, report) = verify_bundle(&bundle).expect("verify");
    assert_eq!(code, 31);
    assert!(
        report
            .errors
            .iter()
            .any(|e| e.code == "DB_ENCRYPTION_MISMATCH")
    );
}

#[test]
fn verifier_accepts_deterministic_zip_bundle() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let folder = root.join("bundle11");
    std::fs::create_dir_all(folder.join("db")).expect("mkdir db");
    std::fs::create_dir_all(folder.join("store/objects/aa")).expect("mkdir objects");
    std::fs::write(folder.join("db/knowledge.sqlite"), b"db").expect("write db");

    let payload = b"zip-object";
    let object_hash = blake3_hex_prefixed(payload);
    std::fs::write(
        folder.join(format!("store/objects/aa/{}", object_hash)),
        payload,
    )
    .expect("write object");

    let mut manifest = base_manifest(blake3_hex_prefixed(b"db"));
    manifest["packaging"]["format"] = serde_json::json!("zip");
    manifest["objects"] = serde_json::json!([{
        "relative_path": format!("store/objects/aa/{}", object_hash),
        "hash": object_hash,
        "storage_hash": blake3_hex_prefixed(payload),
        "encrypted": false,
        "bytes": payload.len()
    }]);
    write_manifest(&folder, &manifest);

    let zip_path = root.join("bundle11.zip");
    let zip_file = std::fs::File::create(&zip_path).expect("create zip");
    let mut writer = zip::ZipWriter::new(zip_file);
    let fixed = zip::DateTime::from_date_and_time(1980, 1, 1, 0, 0, 0).expect("fixed");
    let options = zip::write::FileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .last_modified_time(fixed)
        .unix_permissions(0o644);

    for rel in [
        "db/knowledge.sqlite",
        "manifest.json",
        &format!("store/objects/aa/{}", object_hash),
    ] {
        writer.start_file(rel, options).expect("start file");
        let bytes = std::fs::read(folder.join(rel)).expect("read source");
        std::io::Write::write_all(&mut writer, &bytes).expect("write zip bytes");
    }
    writer.finish().expect("finish zip");

    let (code, report) = verify_bundle(&zip_path).expect("verify zip");
    assert_eq!(code, 0);
    assert_eq!(report.status, "ok");
}

#[test]
fn verifier_sync_head_accepts_v2_with_trust() {
    let payload = serde_json::json!({
        "schema_version": 2,
        "snapshot_id": "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "manifest_hash": "blake3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "created_at_ms": 100,
        "trust": {
            "model": "passphrase_v1",
            "fingerprint": "blake3:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
            "updated_at_ms": 100
        }
    });
    let bytes = serde_json::to_vec(&payload).expect("sync head bytes");
    verify_sync_head_payload(&bytes).expect("sync head should validate");
}

#[test]
fn verifier_sync_head_rejects_v2_without_trust() {
    let payload = serde_json::json!({
        "schema_version": 2,
        "snapshot_id": "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "manifest_hash": "blake3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "created_at_ms": 100
    });
    let bytes = serde_json::to_vec(&payload).expect("sync head bytes");
    let err = verify_sync_head_payload(&bytes).expect_err("sync head should fail schema");
    assert_eq!(err.code, "KC_VERIFY_FAILED");
}
