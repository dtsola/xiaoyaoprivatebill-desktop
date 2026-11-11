//! 审计日志：JSONL 追加写与读取。
//!
//! 格式见技术方案 03 §8.7；永不自动清理，Token 与完整路径不得写入。

use std::fs::OpenOptions;
use std::io::Write;

use bill_core::DataPaths;
use serde_json::{json, Value};

/// 默认读取条数。
pub const DEFAULT_LIMIT: usize = 50;
/// 读取条数上限。
pub const MAX_LIMIT: usize = 500;

/// 追加一行审计记录。
pub fn append(paths: &DataPaths, entry: &Value) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(paths.audit_file())
        .map_err(|error| format!("打开审计日志失败: {error}"))?;
    let mut line =
        serde_json::to_string(entry).map_err(|error| format!("序列化审计记录失败: {error}"))?;
    line.push('\n');
    file.write_all(line.as_bytes())
        .map_err(|error| format!("写入审计日志失败: {error}"))
}

/// 读取最近 `limit` 条；按 `actor.tool` 与 `since` 过滤。
pub fn read(paths: &DataPaths, limit: usize, tool: Option<&str>, since: Option<&str>) -> Value {
    let limit = limit.clamp(1, MAX_LIMIT);
    let Ok(text) = std::fs::read_to_string(paths.audit_file()) else {
        return json!({ "entries": [] });
    };

    let mut entries: Vec<Value> = text
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter(|entry| match tool {
            Some(tool) => {
                entry
                    .get("actor")
                    .and_then(|actor| actor.get("tool"))
                    .and_then(Value::as_str)
                    == Some(tool)
            }
            None => true,
        })
        .filter(|entry| match since {
            Some(since) => entry
                .get("ts")
                .and_then(Value::as_str)
                .is_some_and(|ts| ts >= since),
            None => true,
        })
        .collect();

    if entries.len() > limit {
        entries.drain(..entries.len() - limit);
    }
    json!({ "entries": entries })
}

/// 写入一条审计记录（intent / result）。
pub fn record(
    paths: &DataPaths,
    tool: &str,
    phase: &str,
    result: &str,
    target: Value,
    error: Option<&str>,
) -> Result<(), String> {
    let entry = json!({
        "ts": chrono::Local::now().to_rfc3339(),
        "event": tool,
        "phase": phase,
        "actor": { "kind": "mcp", "tool": tool },
        "request_id": crate::token::short_id(),
        "target": target,
        "result": result,
        "error": error,
        "app_version": env!("CARGO_PKG_VERSION"),
        "schema_version": 1,
    });
    append(paths, &entry)
}
