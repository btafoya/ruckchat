//! RuckChat desktop application library.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::generate_context;

/// Runs the Tauri application with the default shell plugin.
///
/// # Panics
///
/// Panics if the Tauri application fails to start. This is unrecoverable;
/// the process exits.
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .run(generate_context!())
        .expect("error while running tauri application");
}
