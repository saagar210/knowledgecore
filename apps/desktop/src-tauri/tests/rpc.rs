use apps_desktop_tauri::rpc::{vault_init_rpc, RpcResponse, VaultInitReq};

#[test]
fn rpc_envelope_success_shape() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let response = vault_init_rpc(
        VaultInitReq {
            vault_path: root.to_string_lossy().to_string(),
            vault_slug: "demo".to_string(),
        },
        1,
    );

    match response {
        RpcResponse::Ok { ok, data } => {
            assert!(ok);
            assert!(!data.vault_id.is_empty());
        }
        RpcResponse::Err { .. } => panic!("expected success response"),
    }
}
