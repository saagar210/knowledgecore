use apps_desktop_tauri::rpc::{AskQuestionReq, SearchQueryReq, VaultInitReq};

#[test]
fn rpc_schema_requires_now_ms_on_deterministic_requests() {
    let missing_vault_now = serde_json::json!({
        "vault_path": "/tmp/vault",
        "vault_slug": "demo"
    });
    let missing_search_now = serde_json::json!({
        "vault_path": "/tmp/vault",
        "query": "foo"
    });
    let missing_ask_now = serde_json::json!({
        "vault_path": "/tmp/vault",
        "question": "What happened?"
    });

    assert!(serde_json::from_value::<VaultInitReq>(missing_vault_now).is_err());
    assert!(serde_json::from_value::<SearchQueryReq>(missing_search_now).is_err());
    assert!(serde_json::from_value::<AskQuestionReq>(missing_ask_now).is_err());
}

#[test]
fn rpc_schema_rejects_unknown_fields() {
    let req = serde_json::json!({
        "vault_path": "/tmp/vault",
        "query": "foo",
        "now_ms": 123,
        "extra": "nope"
    });
    assert!(serde_json::from_value::<SearchQueryReq>(req).is_err());
}
