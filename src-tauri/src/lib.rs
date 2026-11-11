//! 小遥账单桌面版 Tauri 壳。

mod commands;
mod mcp_state;
mod tray;

use std::sync::Arc;

use bill_core::DataPaths;
use mcp_state::McpState;
use tauri::Manager;

pub fn run() {
    let app = tauri::Builder::default()
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
            // 数据目录固定为 <平台数据目录>/xiaoyaobill-desktop，与 identifier 解耦。
            let paths = DataPaths::from_data_root(&app.path().data_dir()?);
            paths.ensure_layout()?;

            let runtime = Arc::new(bill_api::Runtime::new(paths.clone()));
            app.manage(runtime.clone());
            let mcp_state = McpState::new(runtime, &paths);
            let start_mcp = mcp_state.enabled();
            app.manage(mcp_state);

            // 设置里已启用 MCP 时异步启动，失败只记录 last_error，不阻断启动。
            if start_mcp {
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    let state = handle.state::<McpState>();
                    if let Ok(token) = bill_mcp::token::ensure_token(&state.runtime.paths) {
                        let outcome = state.manager.start(&token).await;
                        let message = outcome.err().map(|error| error.to_string());
                        let _ = state.update_mcp(|mcp| {
                            if message.is_some() {
                                mcp.enabled = false;
                            }
                            mcp.last_error = message;
                        });
                    }
                });
            }

            tray::create_tray(app)?;

            // 首帧后再显示，避免白屏。
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::api_request,
            commands::mcp_status,
            commands::mcp_set_enabled,
            commands::mcp_config,
            commands::mcp_rotate_token,
            commands::desktop_settings,
            commands::set_close_to_tray,
            commands::open_data_dir,
            commands::storage_status,
            commands::open_storage_dir
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let state = window.state::<McpState>();
                if state.desktop_settings().close_to_tray {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("启动失败");

    app.run(|app_handle, event| {
        if let tauri::RunEvent::Exit = event {
            let state = app_handle.state::<McpState>();
            tauri::async_runtime::block_on(state.manager.stop());
        }
    });
}
