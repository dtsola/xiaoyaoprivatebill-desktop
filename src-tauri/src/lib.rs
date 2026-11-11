//! 小遥账单桌面版 Tauri 壳。

mod commands;
mod tray;

use bill_core::DataPaths;
use tauri::Manager;

/// 应用级状态。P1 起保存完整的数据目录布局；后续再扩展设置与数据集。
pub struct AppState {
    pub paths: DataPaths,
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
            let paths = DataPaths::new(app.path().app_data_dir()?);
            paths.ensure_layout()?;
            app.manage(AppState { paths });

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
