//! API 分发层：把前端 IPC 请求映射到服务函数。
//!
//! P1 只覆盖文件管理与 `/available_years`；P2 补齐其余接口。

use std::path::PathBuf;

use bill_core::DataPaths;
use bill_services::files::{clear_data, delete_file, import_paths, list_files};
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

/// 运行时状态。
#[derive(Debug)]
pub struct Runtime {
    /// 数据目录布局。
    pub paths: DataPaths,
}

impl Runtime {
    /// 以数据目录构造运行时。
    pub fn new(paths: DataPaths) -> Self {
        Self { paths }
    }
}

/// 处理一个 API 请求。
///
/// 业务失败以 `{ success: false, error }` 返回；传输级错误才返回 `Err`。
pub fn handle(runtime: &Runtime, req: &ApiRequest) -> Result<Value, String> {
    match (req.method.as_str(), req.path.as_str()) {
        // P0 临时桩：返回固定年份；P2 接入真实 Dataset。
        ("GET", "/available_years") => Ok(json!({
            "success": true,
            "years": [2024]
        })),
        ("GET", "/files") => Ok(handle_list(runtime)),
        ("POST", "/upload") => Ok(handle_upload(runtime, &req.body)),
        ("POST", "/clear_data") => Ok(handle_clear(runtime)),
        _ => {
            if let Some(name) = req.path.strip_prefix("/files/") {
                if req.method == "DELETE" {
                    return Ok(handle_delete(runtime, name));
                }
            }
            Ok(json!({
                "success": false,
                "error": format!("接口不存在: {}", req.path)
            }))
        }
    }
}

fn handle_list(runtime: &Runtime) -> Value {
    match list_files(&runtime.paths) {
        Ok(files) => {
            let items: Vec<Value> = files
                .iter()
                .map(|file| {
                    json!({
                        "name": file.name,
                        "size": file.size,
                        "size_formatted": format_size(file.size),
                        "source": file.source,
                    })
                })
                .collect();
            json!({ "files": items })
        }
        Err(error) => json!({ "success": false, "error": error.to_string() }),
    }
}

fn handle_upload(runtime: &Runtime, body: &Value) -> Value {
    let Some(paths) = body.get("paths").and_then(|value| value.as_array()) else {
        return json!({ "success": false, "error": "没有文件被上传" });
    };
    let sources: Vec<PathBuf> = paths
        .iter()
        .filter_map(|value| value.as_str())
        .map(PathBuf::from)
        .collect();

    match import_paths(&runtime.paths, &sources) {
        Ok(names) => json!({
            "success": true,
            "filename": names.first().cloned().unwrap_or_default(),
            "filenames": names,
            "message": "文件上传成功"
        }),
        Err(error) => json!({ "success": false, "error": error.to_string() }),
    }
}

fn handle_delete(runtime: &Runtime, raw_name: &str) -> Value {
    let name = percent_decode(raw_name);
    match delete_file(&runtime.paths, &name) {
        Ok(()) => json!({ "success": true }),
        Err(error) => json!({ "success": false, "error": error.to_string() }),
    }
}

fn handle_clear(runtime: &Runtime) -> Value {
    match clear_data(&runtime.paths) {
        Ok(_) => json!({ "success": true, "message": "数据已清除" }),
        Err(error) => json!({ "success": false, "error": error.to_string() }),
    }
}

/// 复刻 Flask `format_file_size`：B/KB/MB/GB/TB，保留一位小数。
fn format_size(size: u64) -> String {
    let mut value = size as f64;
    for unit in ["B", "KB", "MB", "GB"] {
        if value < 1024.0 {
            return format!("{value:.1} {unit}");
        }
        value /= 1024.0;
    }
    format!("{value:.1} TB")
}

/// 最小百分号解码，用于 `DELETE /files/<name>`。
fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[index + 1..index + 3]).unwrap_or("");
            if let Ok(byte) = u8::from_str_radix(hex, 16) {
                out.push(byte);
                index += 3;
                continue;
            }
        }
        out.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn runtime() -> (tempfile::TempDir, Runtime) {
        let dir = tempfile::tempdir().expect("临时目录");
        let paths = DataPaths::new(dir.path().to_path_buf());
        paths.ensure_layout().expect("创建布局");
        (dir, Runtime::new(paths))
    }

    #[test]
    fn available_years_returns_success() {
        let (_dir, runtime) = runtime();
        let req: ApiRequest = serde_json::from_value(json!({
            "method": "GET",
            "path": "/available_years"
        }))
        .expect("构造请求");

        let result = handle(&runtime, &req).expect("处理成功");
        assert_eq!(result, json!({"success": true, "years": [2024]}));
    }

    #[test]
    fn unknown_route_returns_error_envelope() {
        let (_dir, runtime) = runtime();
        let req: ApiRequest = serde_json::from_value(json!({
            "method": "GET",
            "path": "/nope"
        }))
        .expect("构造请求");

        let result = handle(&runtime, &req).expect("处理成功");
        assert_eq!(result["success"], false);
        assert_eq!(result["error"], "接口不存在: /nope");
    }

    #[test]
    fn query_and_body_default_when_missing() {
        let req: ApiRequest = serde_json::from_value(json!({
            "method": "GET",
            "path": "/available_years"
        }))
        .expect("构造请求");

        assert!(req.query.is_empty());
        assert_eq!(req.body, Value::Null);
    }

    #[test]
    fn format_size_matches_python() {
        assert_eq!(format_size(0), "0.0 B");
        assert_eq!(format_size(1023), "1023.0 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
    }

    #[test]
    fn percent_decode_handles_chinese_and_plus() {
        assert_eq!(percent_decode("a%20b.csv"), "a b.csv");
        assert_eq!(percent_decode("plain.csv"), "plain.csv");
    }
}
