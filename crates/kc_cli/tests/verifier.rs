use kc_cli::verifier::verify_bundle;
use kc_core::hashing::blake3_hex_prefixed;

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

    let manifest = serde_json::json!({
        "manifest_version": 1,
        "vault_id": "123e4567-e89b-12d3-a456-426614174000",
        "schema_versions": {
            "vault": 1,
            "locator": 1,
            "app_error": 1,
            "rpc": 1
        },
        "chunking_config_hash": "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "db": {
            "relative_path": "db/knowledge.sqlite",
            "hash": blake3_hex_prefixed(db_bytes)
        },
        "objects": [
            {
                "relative_path": format!("store/objects/aa/{}", obj_hash),
                "hash": obj_hash,
                "bytes": obj_bytes.len()
            }
        ],
        "indexes": {}
    });

    std::fs::write(bundle.join("manifest.json"), serde_json::to_vec(&manifest).expect("json"))
        .expect("write manifest");

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

    let manifest = serde_json::json!({
        "manifest_version": 1,
        "vault_id": "123e4567-e89b-12d3-a456-426614174000",
        "schema_versions": {
            "vault": 1,
            "locator": 1,
            "app_error": 1,
            "rpc": 1
        },
        "chunking_config_hash": "blake3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "db": {
            "relative_path": "db/knowledge.sqlite",
            "hash": "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        },
        "objects": [],
        "indexes": {}
    });
    std::fs::write(bundle.join("manifest.json"), serde_json::to_vec(&manifest).expect("json"))
        .expect("write manifest");

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
    std::fs::write(bundle.join("manifest.json"), serde_json::to_vec(&manifest).expect("json"))
        .expect("write manifest");

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

    let obj_hash_1 = "blake3:1111111111111111111111111111111111111111111111111111111111111111";
    let obj_hash_2 = "blake3:2222222222222222222222222222222222222222222222222222222222222222";

    let manifest = serde_json::json!({
        "manifest_version": 1,
        "vault_id": "123e4567-e89b-12d3-a456-426614174000",
        "schema_versions": {
            "vault": 1,
            "locator": 1,
            "app_error": 1,
            "rpc": 1
        },
        "chunking_config_hash": "blake3:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        "db": {
            "relative_path": "db/knowledge.sqlite",
            "hash": "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        },
        "objects": [
            { "relative_path": "store/objects/aa/obj2", "hash": obj_hash_2, "bytes": 10 },
            { "relative_path": "store/objects/aa/obj1", "hash": obj_hash_1, "bytes": 10 }
        ],
        "indexes": {}
    });
    std::fs::write(bundle.join("manifest.json"), serde_json::to_vec(&manifest).expect("json"))
        .expect("write manifest");

    let (_code, report) = verify_bundle(&bundle).expect("verify");
    let sorted = report.errors.clone();
    let mut expected = sorted.clone();
    expected.sort_by(|a, b| a.code.cmp(&b.code).then(a.path.cmp(&b.path)));
    assert_eq!(sorted.iter().map(|e| (&e.code, &e.path)).collect::<Vec<_>>(), expected.iter().map(|e| (&e.code, &e.path)).collect::<Vec<_>>());
}

#[test]
fn verifier_checks_vector_index_files() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let bundle = root.join("bundle5");
    std::fs::create_dir_all(bundle.join("db")).expect("mkdir db");
    std::fs::create_dir_all(bundle.join("index/vectors")).expect("mkdir vectors");
    std::fs::write(bundle.join("db/knowledge.sqlite"), b"db").expect("write db");
    std::fs::write(bundle.join("index/vectors/a.vec"), b"aaa").expect("write vec");

    let manifest = serde_json::json!({
        "manifest_version": 1,
        "vault_id": "123e4567-e89b-12d3-a456-426614174000",
        "schema_versions": {
            "vault": 1,
            "locator": 1,
            "app_error": 1,
            "rpc": 1
        },
        "chunking_config_hash": "blake3:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        "db": {
            "relative_path": "db/knowledge.sqlite",
            "hash": blake3_hex_prefixed(b"db")
        },
        "objects": [],
        "indexes": {
            "vectors": [{
                "relative_path": "index/vectors/a.vec",
                "hash": blake3_hex_prefixed(b"aaa"),
                "bytes": 3
            }]
        }
    });
    std::fs::write(bundle.join("manifest.json"), serde_json::to_vec(&manifest).expect("json"))
        .expect("write manifest");

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

    let missing_hash = "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let manifest = serde_json::json!({
        "manifest_version": 1,
        "vault_id": "123e4567-e89b-12d3-a456-426614174000",
        "schema_versions": {
            "vault": 1,
            "locator": 1,
            "app_error": 1,
            "rpc": 1
        },
        "chunking_config_hash": "blake3:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        "db": {
            "relative_path": "db/knowledge.sqlite",
            "hash": blake3_hex_prefixed(b"db")
        },
        "objects": [{
            "relative_path": "store/objects/aa/missing",
            "hash": missing_hash,
            "bytes": 1
        }],
        "indexes": {}
    });
    std::fs::write(bundle.join("manifest.json"), serde_json::to_vec(&manifest).expect("json"))
        .expect("write manifest");

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

    let manifest = serde_json::json!({
        "manifest_version": 1,
        "vault_id": "123e4567-e89b-12d3-a456-426614174000",
        "schema_versions": {
            "vault": 1,
            "locator": 1,
            "app_error": 1,
            "rpc": 1
        },
        "chunking_config_hash": "blake3:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        "db": {
            "relative_path": "db/knowledge.sqlite",
            "hash": blake3_hex_prefixed(b"db")
        },
        "objects": [{
            "relative_path": "store/objects/aa/x",
            "hash": "blake3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "bytes": 6
        }],
        "indexes": {}
    });
    std::fs::write(bundle.join("manifest.json"), serde_json::to_vec(&manifest).expect("json"))
        .expect("write manifest");

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

    let manifest = serde_json::json!({
        "manifest_version": 1,
        "vault_id": "123e4567-e89b-12d3-a456-426614174000",
        "schema_versions": {
            "vault": 1,
            "locator": 1,
            "app_error": 1,
            "rpc": 1
        },
        "chunking_config_hash": "blake3:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        "db": {
            "relative_path": "db/knowledge.sqlite",
            "hash": blake3_hex_prefixed(b"db")
        },
        "objects": [{
            "relative_path": "store/objects/aa/dir_as_object",
            "hash": "blake3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "bytes": 1
        }],
        "indexes": {}
    });
    std::fs::write(bundle.join("manifest.json"), serde_json::to_vec(&manifest).expect("json"))
        .expect("write manifest");

    let (code, report) = verify_bundle(&bundle).expect("verify");
    assert_eq!(code, 60);
    assert!(report.errors.iter().any(|e| e.code == "INTERNAL_ERROR"));
}
