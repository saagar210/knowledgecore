mod rpc;

use serde_json::Value;

fn dispatch(cmd: &str, req: Value) -> Value {
    match cmd {
        "vault_init" => serde_json::to_value(rpc::vault_init_rpc(
            serde_json::from_value(req).expect("vault_init req"),
        ))
        .expect("serialize vault_init"),
        "vault_open" => {
            serde_json::to_value(rpc::vault_open_rpc(serde_json::from_value(req).expect("vault_open req")))
                .expect("serialize vault_open")
        }
        "ingest_scan_folder" => serde_json::to_value(rpc::ingest_scan_folder_rpc(
            serde_json::from_value(req).expect("ingest_scan_folder req"),
        ))
        .expect("serialize ingest_scan_folder"),
        "ingest_inbox_start" => serde_json::to_value(rpc::ingest_inbox_start_rpc(
            serde_json::from_value(req).expect("ingest_inbox_start req"),
        ))
        .expect("serialize ingest_inbox_start"),
        "ingest_inbox_stop" => serde_json::to_value(rpc::ingest_inbox_stop_rpc(
            serde_json::from_value(req).expect("ingest_inbox_stop req"),
        ))
        .expect("serialize ingest_inbox_stop"),
        "search_query" => {
            serde_json::to_value(rpc::search_query_rpc(serde_json::from_value(req).expect("search_query req")))
                .expect("serialize search_query")
        }
        "locator_resolve" => serde_json::to_value(rpc::locator_resolve_rpc(
            serde_json::from_value(req).expect("locator_resolve req"),
        ))
        .expect("serialize locator_resolve"),
        "export_bundle" => serde_json::to_value(rpc::export_bundle_rpc(
            serde_json::from_value(req).expect("export_bundle req"),
        ))
        .expect("serialize export_bundle"),
        "verify_bundle" => {
            serde_json::to_value(rpc::verify_bundle_rpc(serde_json::from_value(req).expect("verify_bundle req")))
                .expect("serialize verify_bundle")
        }
        "ask_question" => serde_json::to_value(rpc::ask_question_rpc(
            serde_json::from_value(req).expect("ask_question req"),
        ))
        .expect("serialize ask_question"),
        "events_list" => serde_json::to_value(rpc::events_list_rpc(
            serde_json::from_value(req).expect("events_list req"),
        ))
        .expect("serialize events_list"),
        "jobs_list" => {
            serde_json::to_value(rpc::jobs_list_rpc(serde_json::from_value(req).expect("jobs_list req")))
                .expect("serialize jobs_list")
        }
        _ => serde_json::to_value(rpc::RpcResponse::<Value>::err(kc_core::app_error::AppError::new(
            "KC_RPC_UNKNOWN_COMMAND",
            "rpc",
            "unknown rpc command",
            false,
            serde_json::json!({ "command": cmd }),
        )))
        .expect("serialize rpc error"),
    }
}

fn main() {
    let snapshot = dispatch("jobs_list", serde_json::json!({ "vault_path": "." }));
    println!("kc_desktop_tauri rpc runtime: {}", snapshot);
}
