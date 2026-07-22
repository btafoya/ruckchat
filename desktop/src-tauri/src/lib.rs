//! RuckChat desktop application library.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{
    Manager, generate_context,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{TrayIconBuilder, TrayIconEvent},
};
use tauri_plugin_deep_link::DeepLinkExt;

/// Sets up the system tray icon with a show/quit menu.
fn setup_tray(app: &tauri::AppHandle) -> Result<(), tauri::Error> {
    let quit_i = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
    let show_i = MenuItemBuilder::with_id("show", "Show").build(app)?;
    let menu = MenuBuilder::new(app).items(&[&quit_i, &show_i]).build()?;

    TrayIconBuilder::with_id("main-tray")
        .menu(&menu)
        .on_menu_event(
            move |app: &tauri::AppHandle, event| match event.id().as_ref() {
                "quit" => app.exit(0),
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                _ => {}
            },
        )
        .on_tray_icon_event(|tray, event| {
            if let (
                TrayIconEvent::Click {
                    button: tauri::tray::MouseButton::Left,
                    button_state: tauri::tray::MouseButtonState::Up,
                    ..
                },
                Some(window),
            ) = (event, tray.app_handle().get_webview_window("main"))
            {
                let _ = window.show();
                let _ = window.set_focus();
            }
        })
        .build(app)?;

    Ok(())
}

/// Updates the tray tooltip with an unread count.
#[tauri::command]
fn set_unread_count(app_handle: tauri::AppHandle, count: u64) {
    if let Some(tray) = app_handle.tray_by_id("main-tray") {
        let tooltip = if count == 0 {
            String::from("RuckChat")
        } else if count == 1 {
            String::from("RuckChat - 1 unread")
        } else {
            format!("RuckChat - {count} unread")
        };
        let _ = tray.set_tooltip(Some(&tooltip));
    }
}

/// Returns the current platform deep-link URLs on startup.
#[tauri::command]
async fn get_deep_link_url(app_handle: tauri::AppHandle) -> Option<String> {
    if let Ok(Some(urls)) = app_handle.deep_link().get_current() {
        return urls.into_iter().next().map(|url| url.to_string());
    }
    None
}

/// Runs the Tauri application with plugins, tray, and deep-link support.
///
/// # Panics
///
/// Panics if the Tauri application fails to start. This is unrecoverable;
/// the process exits.
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_deep_link::init())
        .setup(|app| {
            setup_tray(app.handle())?;

            #[cfg(desktop)]
            {
                let _ = app.deep_link().register_all();
            }

            app.deep_link().on_open_url(|_event| {
                #[cfg(debug_assertions)]
                eprintln!("deep-link opened: {:?}", _event.urls());
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            set_unread_count,
            get_deep_link_url
        ])
        .run(generate_context!())
        .expect("error while running tauri application");
}
