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
        ]
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
        "db": {
            "relative_path": "db/knowledge.sqlite",
            "hash": "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        },
        "objects": []
    });
    std::fs::write(bundle.join("manifest.json"), serde_json::to_vec(&manifest).expect("json"))
        .expect("write manifest");

    let (code, report) = verify_bundle(&bundle).expect("verify");
    assert_eq!(code, 31);
    assert_eq!(report.status, "failed");
}
