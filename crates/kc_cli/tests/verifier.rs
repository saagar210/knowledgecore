use kc_cli::verifier::verify_bundle;
use kc_core::hashing::blake3_hex_prefixed;

fn base_manifest(db_hash: String) -> serde_json::Value {
    serde_json::json!({
        "manifest_version": 1,
        "vault_id": "123e4567-e89b-12d3-a456-426614174000",
        "schema_versions": {
            "vault": 2,
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
