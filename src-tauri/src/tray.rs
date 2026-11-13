//! 系统托盘。

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{App, AppHandle, Emitter, Manager};
use tauri_plugin_opener::OpenerExt;

use crate::mcp_state::McpState;
use crate::shell_i18n::{normalize_locale, shell_labels, ShellLabels};

/// 托盘图标固定 ID，语言切换时用它找回托盘并重建菜单。
const TRAY_ID: &str = "main-tray";

pub fn create_tray(app: &App) -> tauri::Result<()> {
    let labels = shell_labels(&app.state::<McpState>().shell_locale());
    let enabled = app.state::<McpState>().enabled();
    let (menu, mcp) = build_menu(app.handle(), &labels, enabled)?;
    app.state::<McpState>().set_tray_item(mcp);

    let Some(icon) = app.default_window_icon().cloned() else {
        // 图标缺失时不影响主窗口启动。
        return Ok(());
    };

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .tooltip(labels.app_name)
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
            "mcp" => toggle_mcp(app.clone()),
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;

    Ok(())
}

fn build_menu(
    app: &AppHandle,
    labels: &ShellLabels,
    enabled: bool,
) -> tauri::Result<(Menu<tauri::Wry>, MenuItem<tauri::Wry>)> {
    let show = MenuItem::with_id(app, "show", labels.show, true, None::<&str>)?;
    let data_dir = MenuItem::with_id(app, "data_dir", labels.data_dir, true, None::<&str>)?;
    let mcp_label = if enabled {
        labels.mcp_on
    } else {
        labels.mcp_off
    };
    let mcp = MenuItem::with_id(app, "mcp", mcp_label, true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", labels.quit, true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &data_dir, &mcp, &quit])?;
    Ok((menu, mcp))
}

/// 按语言刷新托盘菜单、tooltip 与主窗口标题。
///
/// 前端在启动和切换语言时调用；语言归一失败时回退中文。
pub fn apply_shell_language(app: &AppHandle, locale: &str) {
    let locale = normalize_locale(locale);
    let labels = shell_labels(locale);

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_title(labels.window_title);
    }

    let state = app.state::<McpState>();
    state.set_shell_locale(locale);

    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };
    let _ = tray.set_tooltip(Some(labels.app_name));
    if let Ok((menu, mcp)) = build_menu(app, &labels, state.enabled()) {
        let _ = tray.set_menu(Some(menu));
        state.set_tray_item(mcp);
    }
}

fn show_main(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn toggle_mcp(handle: AppHandle) {
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

        let locale = state.shell_locale();
        apply_shell_language(&handle, &locale);
        let _ = handle.emit("desktop://mcp-status", state.status());
    });
}
