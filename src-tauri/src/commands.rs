//! Tauri IPC 命令。

use std::sync::Arc;

use serde_json::{json, Value};
use tauri::State;

use crate::mcp_state::McpState;

/// 通用 API 入口：前端通过 `invoke('api_request', { req })` 调用。
///
/// 业务失败以 `{ success: false, error }` 返回；只有传输级错误才返回 `Err`。
#[tauri::command]
pub async fn api_request(
    runtime: State<'_, Arc<bill_api::Runtime>>,
    req: bill_api::ApiRequest,
) -> Result<Value, String> {
    bill_api::handle(runtime.inner().as_ref(), &req)
}

/// MCP 状态。
#[tauri::command]
pub async fn mcp_status(state: State<'_, McpState>) -> Result<Value, String> {
    Ok(state.status())
}

/// 开启/关闭 MCP 服务。
#[tauri::command]
pub async fn mcp_set_enabled(state: State<'_, McpState>, enabled: bool) -> Result<Value, String> {
    if enabled {
        let token = bill_mcp::token::ensure_token(&state.runtime.paths)?;
        state.update_mcp(|mcp| mcp.enabled = true)?;
        match state.manager.start(&token).await {
            Ok(()) => {
                state.update_mcp(|mcp| mcp.last_error = None)?;
            }
            Err(error) => {
                let message = error.to_string();
                state.update_mcp(|mcp| {
                    mcp.enabled = false;
                    mcp.last_error = Some(message);
                })?;
            }
        }
    } else {
        state.manager.stop().await;
        state.update_mcp(|mcp| {
            mcp.enabled = false;
            mcp.last_error = None;
        })?;
    }
    Ok(state.status())
}

/// 客户端配置：端点 + Token。
#[tauri::command]
pub async fn mcp_config(state: State<'_, McpState>) -> Result<Value, String> {
    let token = bill_mcp::token::ensure_token(&state.runtime.paths)?;
    let mcp = state.mcp_settings();
    Ok(json!({
        "endpoint": format!("http://127.0.0.1:{}/mcp", mcp.port),
        "token": token,
    }))
}

/// 轮换 Token；运行中则重启服务。
#[tauri::command]
pub async fn mcp_rotate_token(state: State<'_, McpState>) -> Result<Value, String> {
    let was_running = state.manager.is_running();
    if was_running {
        state.manager.stop().await;
    }
    let token = bill_mcp::token::rotate_token(&state.runtime.paths)?;
    if was_running {
        state
            .manager
            .start(&token)
            .await
            .map_err(|error| error.to_string())?;
    }
    let mcp = state.mcp_settings();
    Ok(json!({
        "endpoint": format!("http://127.0.0.1:{}/mcp", mcp.port),
        "token": token,
    }))
}

/// 桌面设置：关闭行为与数据目录。
#[tauri::command]
pub async fn desktop_settings(state: State<'_, McpState>) -> Result<Value, String> {
    let desktop = state.desktop_settings();
    Ok(json!({
        "close_to_tray": desktop.close_to_tray,
        "data_dir": state.data_dir(),
    }))
}

/// 更新「关闭到托盘」开关。
#[tauri::command]
pub async fn set_close_to_tray(state: State<'_, McpState>, enabled: bool) -> Result<Value, String> {
    state.update_desktop(|desktop| desktop.close_to_tray = enabled)?;
    Ok(json!({ "close_to_tray": enabled }))
}

/// 用系统文件管理器打开数据目录。
#[tauri::command]
pub async fn open_data_dir(
    app: tauri::AppHandle,
    state: State<'_, McpState>,
) -> Result<(), String> {
    tauri_plugin_opener::OpenerExt::opener(&app)
        .open_path(state.data_dir(), None::<&str>)
        .map_err(|error| error.to_string())
}

/// 打开回收站或快照目录。
#[tauri::command]
pub async fn open_storage_dir(
    app: tauri::AppHandle,
    state: State<'_, McpState>,
    kind: String,
) -> Result<(), String> {
    let paths = &state.runtime.paths;
    let dir = match kind.as_str() {
        "trash" => paths.trash_dir(),
        "snapshots" => paths.snapshots_dir(),
        _ => return Err("未知目录类型".to_owned()),
    };
    std::fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    tauri_plugin_opener::OpenerExt::opener(&app)
        .open_path(dir.display().to_string(), None::<&str>)
        .map_err(|error| error.to_string())
}

/// 存储概览：回收站条目、快照数、审计日志大小。
#[tauri::command]
pub async fn storage_status(state: State<'_, McpState>) -> Result<Value, String> {
    let paths = &state.runtime.paths;
    let count_entries = |dir: std::path::PathBuf| {
        std::fs::read_dir(dir)
            .map(|entries| {
                entries
                    .filter_map(|entry| entry.ok())
                    .filter(|entry| entry.path().is_dir())
                    .count()
            })
            .unwrap_or(0)
    };
    let audit_bytes = std::fs::metadata(paths.audit_file())
        .map(|meta| meta.len())
        .unwrap_or(0);

    Ok(json!({
        "trash_count": count_entries(paths.trash_dir()),
        "snapshot_count": count_entries(paths.snapshots_dir()),
        "audit_bytes": audit_bytes,
        "audit_path": paths.audit_file().display().to_string(),
    }))
}
