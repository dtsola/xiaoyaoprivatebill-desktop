//! 系统托盘。

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{App, Emitter, Manager};
use tauri_plugin_opener::OpenerExt;

use crate::mcp_state::McpState;

pub fn create_tray(app: &App) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
    let data_dir = MenuItem::with_id(app, "data_dir", "打开数据目录", true, None::<&str>)?;
    let mcp_label = if app.state::<McpState>().enabled() {
        "关闭 MCP 服务"
    } else {
        "开启 MCP 服务"
    };
    let mcp = MenuItem::with_id(app, "mcp", mcp_label, true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    app.state::<McpState>().set_tray_item(mcp.clone());
    let menu = Menu::with_items(app, &[&show, &data_dir, &mcp, &quit])?;

    let Some(icon) = app.default_window_icon().cloned() else {
        // 图标缺失时不影响主窗口启动。
        return Ok(());
    };

    TrayIconBuilder::new()
        .icon(icon)
        .tooltip("小遥账单桌面版")
        .menu(&menu)
        // 左键直接显示主窗口；右键（默认）呼出菜单，避免左键无响应。
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main(tray.app_handle());
            }
        })
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "show" => show_main(app),
            "data_dir" => {
                let dir = app.state::<McpState>().data_dir();
                let _ = app.opener().open_path(dir, None::<&str>);
            }
            "mcp" => toggle_mcp(app, mcp.clone()),
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;

    Ok(())
}

fn show_main(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn toggle_mcp(app: &tauri::AppHandle, item: MenuItem<tauri::Wry>) {
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let state = handle.state::<McpState>();
        let next = !state.enabled();
        if next {
            if let Ok(token) = bill_mcp::token::ensure_token(&state.runtime.paths) {
                let outcome = state.manager.start(&token).await;
                let message = outcome.err().map(|error| error.to_string());
                let _ = state.update_mcp(|mcp| {
                    mcp.enabled = message.is_none();
                    mcp.last_error = message;
                });
            }
        } else {
            state.manager.stop().await;
            let _ = state.update_mcp(|mcp| {
                mcp.enabled = false;
                mcp.last_error = None;
            });
        }
        let label = if state.enabled() {
            "关闭 MCP 服务"
        } else {
            "开启 MCP 服务"
        };
        let _ = item.set_text(label);
        let _ = handle.emit("desktop://mcp-status", state.status());
    });
}
