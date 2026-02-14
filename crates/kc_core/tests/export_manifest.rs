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

    let hashes: Vec<String> = objects
        .iter()
        .map(|o| o.get("hash").and_then(|v| v.as_str()).unwrap_or_default().to_string())
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
    let expected_chunking_hash = hash_chunking_config(&default_chunking_config_v1())
        .expect("hash default chunking config");
    assert_eq!(chunking_hash, expected_chunking_hash.0);
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
        },
        124,
    )
    .expect("export");

    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(bundle.join("manifest.json")).expect("read manifest"))
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
