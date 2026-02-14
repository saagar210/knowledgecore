#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use apps_desktop_tauri::commands;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::vault_init,
            commands::vault_open,
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
        ])
        .run(tauri::generate_context!())
        .expect("failed to run tauri app");
}
