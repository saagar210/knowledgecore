#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use apps_desktop_tauri::commands;

fn main() {
    #[cfg(feature = "phase_l_preview")]
    let builder = tauri::Builder::default().invoke_handler(tauri::generate_handler![
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
        commands::preview_status,
        commands::preview_capability,
    ]);

    #[cfg(not(feature = "phase_l_preview"))]
    let builder = tauri::Builder::default().invoke_handler(tauri::generate_handler![
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
    ]);

    builder.run(tauri::generate_context!()).expect("failed to run tauri app");
}
