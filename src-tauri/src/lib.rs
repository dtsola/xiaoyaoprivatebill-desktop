//! 小遥账单桌面版 Tauri 壳。

mod commands;
mod tray;

use std::path::PathBuf;
use tauri::Manager;

/// 应用级状态。P0 只保存数据目录；P1 会扩展为完整的 AppState。
pub struct AppState {
    pub app_data_dir: PathBuf,
}

pub fn run() {
    tauri::Builder::default()
        // 单实例必须是第一个注册的插件。
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;
            app.manage(AppState { app_data_dir });

            tray::create_tray(app)?;

            // 首帧后再显示，避免白屏。
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![commands::api_request])
        .run(tauri::generate_context!())
        .expect("启动失败");
}
