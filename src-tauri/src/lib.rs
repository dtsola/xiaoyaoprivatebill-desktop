//! 小遥账单桌面版 Tauri 壳。

mod commands;
mod mcp_state;
mod shell_i18n;
mod tray;

use std::sync::Arc;

use bill_core::DataPaths;
use mcp_state::McpState;
use tauri::{Emitter, Manager};

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

            // 已明确选择过语言时，启动即同步托盘与窗口标题；
            // `system` 由前端读取系统语言后调用 apply_shell_language。
            let language = app.state::<McpState>().desktop_settings().language;
            if language != "system" {
                let handle = app.handle().clone();
                tray::apply_shell_language(&handle, &language);
            }

            // 首帧后再显示，避免白屏。
            if let Some(window) = app.get_webview_window("main") {
                // Windows 去掉原生标题栏，改由前端 AppLayout 渲染拖拽区与窗口按钮；
                // macOS 走 titleBarStyle: Overlay，保留原生红绿灯。
                #[cfg(target_os = "windows")]
                let _ = window.set_decorations(false);
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
            commands::open_storage_dir,
            commands::share_qr_svg,
            commands::save_share_poster,
            commands::set_language,
            commands::apply_shell_language
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let state = window.state::<McpState>();
                let desktop = state.desktop_settings();
                if desktop.close_to_tray {
                    api.prevent_close();
                    let _ = window.hide();
                    if !desktop.close_hint_shown {
                        let _ = state.update_desktop(|settings| settings.close_hint_shown = true);
                        let _ = window.emit("desktop://close-to-tray-hint", ());
                    }
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
