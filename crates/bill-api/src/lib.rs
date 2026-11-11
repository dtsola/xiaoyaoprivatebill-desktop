//! API 分发层。
//!
//! P0 只实现 `/available_years` 用于打通 IPC；P2 补齐 27 个接口。

use serde::Deserialize;
use serde_json::{json, Value};

/// 前端 IPC 请求。
#[derive(Debug, Deserialize)]
pub struct ApiRequest {
    pub method: String,
    /// 不含 `/api` 前缀，例如 `/available_years`。
    pub path: String,
    #[serde(default)]
    pub query: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    pub body: Value,
}

/// 处理一个 API 请求。
///
/// 业务失败以 `{ success: false, error }` 返回；传输级错误才返回 `Err`。
pub fn handle(req: &ApiRequest) -> Result<Value, String> {
    match (req.method.as_str(), req.path.as_str()) {
        // P0 临时桩：返回固定年份用于验证 IPC；P1 接入真实 Dataset。
        ("GET", "/available_years") => Ok(json!({
            "success": true,
            "years": [2024]
        })),
        _ => Ok(json!({
            "success": false,
            "error": format!("接口不存在: {}", req.path)
        })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn available_years_returns_success() {
        let req: ApiRequest = serde_json::from_value(json!({
            "method": "GET",
            "path": "/available_years"
        }))
        .unwrap();

        let result = handle(&req).unwrap();
        assert_eq!(result, json!({"success": true, "years": [2024]}));
    }

    #[test]
    fn unknown_route_returns_error_envelope() {
        let req: ApiRequest = serde_json::from_value(json!({
            "method": "GET",
            "path": "/nope"
        }))
        .unwrap();

        let result = handle(&req).unwrap();
        assert_eq!(result["success"], false);
        assert_eq!(result["error"], "接口不存在: /nope");
    }

    #[test]
    fn query_and_body_default_when_missing() {
        let req: ApiRequest = serde_json::from_value(json!({
            "method": "GET",
            "path": "/available_years"
        }))
        .unwrap();

        assert!(req.query.is_empty());
        assert_eq!(req.body, Value::Null);
    }
}
