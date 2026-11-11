//! P4-4 端到端：通过 Streamable HTTP 列出并调用工具。

use std::sync::Arc;

use axum::body::Body;
use axum::http::{header, HeaderMap, Request, StatusCode};
use bill_api::Runtime;
use bill_core::{DataPaths, Dataset};
use bill_mcp::router_with_token;
use serde_json::{json, Value};
use tower::ServiceExt;

const TOKEN: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn runtime() -> Arc<Runtime> {
    let dir = tempfile::tempdir().expect("临时目录");
    let paths = DataPaths::new(dir.path().to_path_buf());
    paths.ensure_layout().expect("创建布局");
    std::mem::forget(dir);
    Arc::new(Runtime::with_dataset(paths, Dataset::default()))
}

async fn post(
    app: &axum::Router,
    session: Option<&str>,
    body: Value,
) -> (StatusCode, HeaderMap, String) {
    let mut builder = Request::builder()
        .method("POST")
        .uri("/mcp")
        .header(header::HOST, "127.0.0.1:17650")
        .header(header::AUTHORIZATION, format!("Bearer {TOKEN}"))
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::ACCEPT, "application/json, text/event-stream");
    if let Some(session) = session {
        builder = builder.header("mcp-session-id", session);
    }
    let response = app
        .clone()
        .oneshot(builder.body(Body::from(body.to_string())).expect("请求"))
        .await
        .expect("响应");

    let status = response.status();
    let headers = response.headers().clone();
    let bytes = axum::body::to_bytes(response.into_body(), 4 * 1024 * 1024)
        .await
        .expect("读取响应体");
    (
        status,
        headers,
        String::from_utf8_lossy(&bytes).into_owned(),
    )
}

/// 从 JSON 或 SSE 响应体中取出第一个 JSON-RPC 消息。
fn parse_message(text: &str) -> Value {
    for line in text.lines() {
        let line = line.trim();
        if let Some(data) = line.strip_prefix("data:") {
            if let Ok(value) = serde_json::from_str::<Value>(data.trim()) {
                return value;
            }
        }
        if line.starts_with('{') {
            if let Ok(value) = serde_json::from_str::<Value>(line) {
                return value;
            }
        }
    }
    panic!("无法解析 MCP 响应: {text}");
}

#[tokio::test]
async fn lists_ten_tools_and_calls_list_bills() {
    let app = router_with_token(runtime(), TOKEN);

    let (status, headers, text) = post(
        &app,
        None,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-03-26",
                "capabilities": {},
                "clientInfo": {"name": "e2e", "version": "0"}
            }
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "initialize 失败: {text}");
    let session = headers
        .get("mcp-session-id")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    assert!(session.is_some(), "initialize 应返回 mcp-session-id");

    let (status, _, text) = post(
        &app,
        session.as_deref(),
        json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "tools/list 失败: {text}");
    let message = parse_message(&text);
    let tools = message["result"]["tools"].as_array().expect("工具列表");

    let mut names: Vec<&str> = tools
        .iter()
        .filter_map(|tool| tool["name"].as_str())
        .collect();
    names.sort_unstable();
    assert_eq!(
        names,
        vec![
            "delete_bill",
            "get_audit_log",
            "get_category_analysis",
            "get_insights",
            "get_monthly_analysis",
            "get_overview",
            "get_summary",
            "get_time_analysis",
            "get_yearly_analysis",
            "import_bill",
            "list_bills",
            "restore_bill",
            "search_transactions",
        ]
    );

    // 只读注解必须存在。
    let list_bills = tools
        .iter()
        .find(|tool| tool["name"] == "list_bills")
        .expect("list_bills");
    assert_eq!(list_bills["annotations"]["readOnlyHint"], json!(true));

    let (status, _, text) = post(
        &app,
        session.as_deref(),
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {"name": "list_bills", "arguments": {}}
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "tools/call 失败: {text}");
    let message = parse_message(&text);
    let content = message["result"]["content"][0]["text"]
        .as_str()
        .expect("文本结果");
    let payload: Value = serde_json::from_str(content).expect("结果是 JSON");
    assert_eq!(payload["files"], json!([]));
    assert_eq!(payload["demo"], json!(false));
}
