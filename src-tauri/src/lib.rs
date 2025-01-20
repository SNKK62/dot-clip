use std::{sync::RwLock, thread, time};

use mouse_position::mouse_position::Mouse;
use serde::{Deserialize, Serialize};
use tauri::tray::TrayIconBuilder;
use tauri::{Manager, WindowEvent};
use tauri_plugin_clipboard_manager::ClipboardExt;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct CursorPosition {
    x: i32,
    y: i32,
}

#[tauri::command]
fn get_cursor_position(app: tauri::AppHandle) -> CursorPosition {
    let m = Mouse::get_mouse_position();
    let pos = match m {
        Mouse::Position { x, y } => CursorPosition { x, y },
        Mouse::Error => CursorPosition { x: 0, y: 0 },
    };
    let previous_state = app.state::<RwLock<AppState>>();
    let mut previous_state = previous_state.write().unwrap();
    previous_state.last_cursor_position = Some(pos.clone());
    pos
}

struct AppState {
    clipboard_history: Vec<String>,
    last_cursor_position: Option<CursorPosition>,
}

#[tauri::command]
fn watch_clipboard(app: tauri::AppHandle) {
    std::thread::spawn(move || loop {
        let mut should_append = false;
        let content = match app.clipboard().read_text() {
            Ok(content) => content,
            Err(e) => {
                println!("Error reading clipboard: {:?}", e);
                thread::sleep(time::Duration::from_secs(1));
                continue;
            }
        };
        let previous_state_atom = app.state::<RwLock<AppState>>();
        {
            let previous_state = previous_state_atom.read().unwrap();
            if previous_state.clipboard_history.is_empty() {
                should_append = true
            } else if *previous_state.clipboard_history.last().unwrap() != content {
                should_append = true;
            }
        }
        if should_append {
            let mut previous_state = previous_state_atom.write().unwrap();
            previous_state.clipboard_history.push(content.clone());
            println!("Clipboard changed: {}", content);
        }
        thread::sleep(time::Duration::from_secs(1));
    });
}

#[tauri::command]
fn get_previous_content(app: tauri::AppHandle) -> Vec<String> {
    let previous_state = app.state::<RwLock<AppState>>();
    let previous_state = previous_state.read().unwrap();
    previous_state.clipboard_history.clone()
}

#[tauri::command]
fn open_submenu(app: tauri::AppHandle) {
    let previous_state = app.state::<RwLock<AppState>>();
    let previous_state = previous_state.read().unwrap();
    let pos = previous_state.last_cursor_position.as_ref().unwrap();
    println!("pos: {:?}", pos);
    let pos = tauri::LogicalPosition {
        x: pos.x + 140,
        y: pos.y + 20,
    };
    let window = app.get_webview_window("sub_menu").unwrap();
    window.set_position(pos).unwrap();
    window.show().unwrap();
    window.set_focus().unwrap()
}

#[tauri::command]
fn close_submenu(app: tauri::AppHandle) {
    let window = app.get_webview_window("sub_menu").unwrap();
    window.hide().unwrap();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {
            // default single instance handler
        }))
        // disable the default menu on macOS
        .enable_macos_default_menu(false)
        .setup(|app| {
            let tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .build(app)?;
            app.manage(RwLock::new(AppState {
                clipboard_history: Vec::new(),
                last_cursor_position: None,
            }));
            // hide the icon in dock on macOS
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let main_window = app.get_webview_window("main_menu").unwrap();
            main_window.hide().unwrap(); // hide the window on start
            let sub_window = app.get_webview_window("sub_menu").unwrap();
            sub_window.hide().unwrap(); // hide the window on start

            // open devtools on debug builds
            #[cfg(debug_assertions)] // only include this code on debug builds
            {
                // main_window.open_devtools();
                // main_window.close_devtools();
                // sub_window.open_devtools();
                // sub_window.close_devtools();
            }
            Ok(())
        })
        .on_window_event(|app, event| {
            if let WindowEvent::Focused(_focused) = event {
                // let previous_state = app.state::<RwLock<AppState>>();
                // let mut previous_state = previous_state.write().unwrap();
                let all_windows = app.windows();
                let visible_windows = all_windows
                    .iter()
                    .filter(move |(_label, win)| win.is_visible().unwrap())
                    .collect::<std::collections::HashMap<_, _>>();

                println!("label: {:?}", app.label());
                // println!("focused_window: {:?}", previous_state.focused_window);
                println!("count: {}", visible_windows.len());

                let main_window = app.get_webview_window("main_menu").unwrap();
                let sub_window = app.get_webview_window("sub_menu").unwrap();

                if visible_windows.len() == 2 {
                    if !main_window.is_focused().unwrap() && !sub_window.is_focused().unwrap() {
                        main_window.hide().unwrap();
                        sub_window.hide().unwrap();
                    }
                } else if visible_windows.len() == 1 && !main_window.is_focused().unwrap() {
                    main_window.hide().unwrap();
                }
            }
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            watch_clipboard,
            get_previous_content,
            get_cursor_position,
            open_submenu,
            close_submenu
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
