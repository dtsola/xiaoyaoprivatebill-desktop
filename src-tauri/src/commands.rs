//! Tauri IPC 命令。

use serde_json::Value;

/// 通用 API 入口：前端通过 `invoke('api_request', { req })` 调用。
///
/// 业务失败以 `{ success: false, error }` 返回；只有传输级错误才返回 `Err`。
#[tauri::command]
pub async fn api_request(
    runtime: tauri::State<'_, bill_api::Runtime>,
    req: bill_api::ApiRequest,
) -> Result<Value, String> {
    bill_api::handle(&runtime, &req)
}
