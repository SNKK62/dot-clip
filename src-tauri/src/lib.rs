use std::{sync::RwLock, thread, time};

use mouse_position::mouse_position::Mouse;
use serde::{Deserialize, Serialize};
use tauri::tray::TrayIconBuilder;
use tauri::{Manager, WindowEvent};
use tauri_plugin_clipboard_manager::ClipboardExt;

use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

// 履歴ファイル名
const HISTORY_FILENAME: &str = "clipboard_history.txt";

// 履歴ファイルのパスを取得
fn get_history_file_path() -> Result<PathBuf, String> {
    let exe_dir = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .parent()
        .ok_or_else(|| "Failed to get application directory".to_string())?
        .to_path_buf();
    println!("exe_dir: {:?}", exe_dir);

    Ok(exe_dir.join(HISTORY_FILENAME))
}

fn save_clipboard_to_history(latest_clipboard_content: String) -> Result<(), String> {
    // クリップボードの内容を取得
    let content = latest_clipboard_content;
    // 空文字列や重複を無視
    if content.trim().is_empty() {
        return Ok(());
    }

    // エスケープ処理 (改行文字を `\n` に変換)
    let escaped_content = content.replace("\n", "\\n");

    // 履歴ファイルのパスを取得
    let history_file_path = get_history_file_path()?;

    // 現在の履歴を読み込む
    let history = load_history(&history_file_path)?;

    // 重複を防ぐため、既存の履歴を確認
    if !history.contains(&escaped_content) {
        // ファイルに追記
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&history_file_path)
            .map_err(|e| e.to_string())?;

        writeln!(file, "{}", escaped_content).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
fn load_history(file_path: &Path) -> Result<Vec<String>, String> {
    // 履歴ファイルが存在しない場合は空のベクタを返す
    if !file_path.exists() {
        return Ok(Vec::new());
    }

    // ファイルを行単位で読み込む
    let file = fs::File::open(file_path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);

    let history = reader
        .lines()
        .map(|line| {
            line.map_err(|e| e.to_string())
                .map(|s| s.replace("\\n", "\n"))
        })
        .collect::<Result<Vec<String>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(history)
}

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
            if let Err(e) = save_clipboard_to_history(content) {
                println!("Error saving clipboard to history: {:?}", e);
            }
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
        .plugin(tauri_plugin_fs::init())
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

            let initial_clipboard_content =
                load_history(&get_history_file_path().unwrap()).unwrap();

            app.manage(RwLock::new(AppState {
                clipboard_history: initial_clipboard_content,
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
