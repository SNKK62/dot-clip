use std::{sync::RwLock, thread, time};

use mouse_position::mouse_position::Mouse;
use serde::{Deserialize, Serialize};
use tauri::tray::TrayIconBuilder;
use tauri::{Manager, WindowEvent};
use tauri_plugin_clipboard_manager::ClipboardExt;

#[derive(Serialize, Deserialize)]
struct CursorPosition {
    x: i32,
    y: i32,
}

#[tauri::command]
fn get_cursor_position() -> CursorPosition {
    let m = Mouse::get_mouse_position();
    match m {
        Mouse::Position { x, y } => CursorPosition { x, y },
        Mouse::Error => CursorPosition { x: 0, y: 0 },
    }
}

struct AppState {
    previous_clipboard: String,
}

#[tauri::command]
fn watch_clipboard(app: tauri::AppHandle) {
    std::thread::spawn(move || loop {
        let mut should_overwrite = false;
        let content = app.clipboard().read_text().unwrap();
        let previous_state_atom = app.state::<RwLock<AppState>>();
        {
            let previous_state = previous_state_atom.read().unwrap();
            if previous_state.previous_clipboard != content {
                should_overwrite = true;
            }
        }
        if should_overwrite {
            let mut previous_state = previous_state_atom.write().unwrap();
            previous_state.previous_clipboard = content.clone();
            println!("Clipboard changed: {}", content);
        }
        thread::sleep(time::Duration::from_secs(1));
    });
}

#[tauri::command]
fn get_previous_content(app: tauri::AppHandle) -> String {
    let previous_state = app.state::<RwLock<AppState>>();
    let previous_state = previous_state.read().unwrap();
    previous_state.previous_clipboard.clone()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {
            // default single instance handler
        }))
        .setup(|app| {
            let tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .build(app)?;
            app.manage(RwLock::new(AppState {
                previous_clipboard: "".to_string(),
            }));
            // open devtools on debug builds
            #[cfg(debug_assertions)] // only include this code on debug builds
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
                window.close_devtools();
                window.hide().unwrap(); // hide the window on start
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                window.hide().unwrap();
            } else if let WindowEvent::Focused(focused) = event {
                if !focused {
                    window.hide().unwrap();
                }
            }
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            watch_clipboard,
            get_previous_content,
            get_cursor_position
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
