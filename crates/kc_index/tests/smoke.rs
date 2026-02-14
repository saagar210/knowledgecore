#[test]
fn index_smoke() {
    let dir = tempfile::tempdir().expect("tempdir");
    assert!(dir.path().exists());
}
