use kc_core::db::open_db;
use kc_core::object_store::{derive_object_store_key, ObjectStore, ObjectStoreEncryptionContext};

#[test]
fn object_store_dedupes_by_content_hash() {
    let temp = tempfile::tempdir().expect("tempdir");
    let db_path = temp.path().join("db/knowledge.sqlite");
    let conn = open_db(&db_path).expect("open db");

    let store = ObjectStore::new(temp.path().join("store/objects"));

    let payload = b"same-bytes";
    let h1 = store.put_bytes(&conn, payload, 1).expect("first put");
    let h2 = store.put_bytes(&conn, payload, 2).expect("second put");

    assert_eq!(h1, h2);
    assert!(store.exists(&h1).expect("exists"));
    assert_eq!(store.get_bytes(&h1).expect("get"), payload);

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM objects WHERE object_hash=?1", [h1.0], |row| row.get(0))
        .expect("count query");
    assert_eq!(count, 1);
}

#[test]
fn object_store_encrypted_round_trip_requires_key() {
    let temp = tempfile::tempdir().expect("tempdir");
    let db_path = temp.path().join("db/knowledge.sqlite");
    let conn = open_db(&db_path).expect("open db");
    let objects_dir = temp.path().join("store/objects");

    let key = derive_object_store_key("passphrase", "salt-0001", 4096, 2, 1).expect("derive key");
    let encrypted_store = ObjectStore::with_encryption(
        objects_dir.clone(),
        ObjectStoreEncryptionContext {
            key,
            key_reference: "vault:test".to_string(),
        },
    );

    let payload = b"secret-bytes";
    let hash = encrypted_store.put_bytes(&conn, payload, 1).expect("put");

    let raw_path = objects_dir
        .join(&hash.0[7..9])
        .join(&hash.0);
    let raw = std::fs::read(raw_path).expect("read raw encrypted object");
    assert!(raw.starts_with(b"KCE1"));

    let plain_store = ObjectStore::new(objects_dir.clone());
    let err = plain_store.get_bytes(&hash).expect_err("read without key should fail");
    assert_eq!(err.code, "KC_ENCRYPTION_REQUIRED");

    let round_trip = encrypted_store.get_bytes(&hash).expect("decrypt with key");
    assert_eq!(round_trip, payload);
}
