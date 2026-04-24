#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

mod desktop_api;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            desktop_api::compare_single,
            desktop_api::scan_directory,
            desktop_api::inspect_single
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
