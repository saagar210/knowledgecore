use crate::rpc;

#[tauri::command]
pub fn vault_init(req: rpc::VaultInitReq) -> rpc::RpcResponse<rpc::VaultInitRes> {
    rpc::vault_init_rpc(req)
}

#[tauri::command]
pub fn vault_open(req: rpc::VaultOpenReq) -> rpc::RpcResponse<rpc::VaultOpenRes> {
    rpc::vault_open_rpc(req)
}

#[tauri::command]
pub fn vault_encryption_status(
    req: rpc::VaultEncryptionStatusReq,
) -> rpc::RpcResponse<rpc::VaultEncryptionStatusRes> {
    rpc::vault_encryption_status_rpc(req)
}

#[tauri::command]
pub fn vault_encryption_enable(
    req: rpc::VaultEncryptionEnableReq,
) -> rpc::RpcResponse<rpc::VaultEncryptionEnableRes> {
    rpc::vault_encryption_enable_rpc(req)
}

#[tauri::command]
pub fn vault_encryption_migrate(
    req: rpc::VaultEncryptionMigrateReq,
) -> rpc::RpcResponse<rpc::VaultEncryptionMigrateRes> {
    rpc::vault_encryption_migrate_rpc(req)
}

#[tauri::command]
pub fn ingest_scan_folder(req: rpc::IngestScanFolderReq) -> rpc::RpcResponse<rpc::IngestScanFolderRes> {
    rpc::ingest_scan_folder_rpc(req)
}

#[tauri::command]
pub fn ingest_inbox_start(req: rpc::IngestInboxStartReq) -> rpc::RpcResponse<rpc::IngestInboxStartRes> {
    rpc::ingest_inbox_start_rpc(req)
}

#[tauri::command]
pub fn ingest_inbox_stop(req: rpc::IngestInboxStopReq) -> rpc::RpcResponse<rpc::IngestInboxStopRes> {
    rpc::ingest_inbox_stop_rpc(req)
}

#[tauri::command]
pub fn search_query(req: rpc::SearchQueryReq) -> rpc::RpcResponse<rpc::SearchQueryRes> {
    rpc::search_query_rpc(req)
}

#[tauri::command]
pub fn locator_resolve(req: rpc::LocatorResolveReq) -> rpc::RpcResponse<rpc::LocatorResolveRes> {
    rpc::locator_resolve_rpc(req)
}

#[tauri::command]
pub fn export_bundle(req: rpc::ExportBundleReq) -> rpc::RpcResponse<rpc::ExportBundleRes> {
    rpc::export_bundle_rpc(req)
}

#[tauri::command]
pub fn verify_bundle(req: rpc::VerifyBundleReq) -> rpc::RpcResponse<rpc::VerifyBundleRes> {
    rpc::verify_bundle_rpc(req)
}

#[tauri::command]
pub fn ask_question(req: rpc::AskQuestionReq) -> rpc::RpcResponse<rpc::AskQuestionRes> {
    rpc::ask_question_rpc(req)
}

#[tauri::command]
pub fn events_list(req: rpc::EventsListReq) -> rpc::RpcResponse<rpc::EventsListRes> {
    rpc::events_list_rpc(req)
}

#[tauri::command]
pub fn jobs_list(req: rpc::JobsListReq) -> rpc::RpcResponse<rpc::JobsListRes> {
    rpc::jobs_list_rpc(req)
}

#[cfg(feature = "phase_l_preview")]
#[tauri::command]
pub fn preview_status(req: rpc::PreviewStatusReq) -> rpc::RpcResponse<rpc::PreviewStatusRes> {
    rpc::preview_status_rpc(req)
}

#[cfg(feature = "phase_l_preview")]
#[tauri::command]
pub fn preview_capability(req: rpc::PreviewCapabilityReq) -> rpc::RpcResponse<rpc::PreviewCapabilityRes> {
    rpc::preview_capability_rpc(req)
}
