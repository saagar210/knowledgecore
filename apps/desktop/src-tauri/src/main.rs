#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use apps_desktop_tauri::commands;

fn main() {
    let builder = tauri::Builder::default().invoke_handler(tauri::generate_handler![
        commands::vault_init,
        commands::vault_open,
        commands::vault_lock_status,
        commands::vault_unlock,
        commands::vault_lock,
        commands::vault_encryption_status,
        commands::vault_encryption_enable,
        commands::vault_encryption_migrate,
        commands::vault_recovery_status,
        commands::vault_recovery_generate,
        commands::vault_recovery_verify,
        commands::ingest_scan_folder,
        commands::ingest_inbox_start,
        commands::ingest_inbox_stop,
        commands::search_query,
        commands::locator_resolve,
        commands::export_bundle,
        commands::verify_bundle,
        commands::ask_question,
        commands::events_list,
        commands::jobs_list,
        commands::sync_status,
        commands::sync_push,
        commands::sync_pull,
        commands::sync_merge_preview,
        commands::lineage_query,
        commands::lineage_query_v2,
        commands::lineage_overlay_add,
        commands::lineage_overlay_remove,
        commands::lineage_overlay_list,
    ]);

    builder
        .run(tauri::generate_context!())
        .expect("failed to run tauri app");
}
