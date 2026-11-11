//! P0 编译验证：确认 axum + rmcp Streamable HTTP 路由可构建、可响应。
//!
//! 这里只验证传输层契约（挂载路径、Host 校验、请求体上限），
//! 14 个 MCP 工具的契约测试在 P4 补齐。

use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use bill_mcp::router;
use tower::ServiceExt;

const LOOPBACK_HOST: &str = "127.0.0.1:17650";

fn runtime() -> std::sync::Arc<bill_api::Runtime> {
    let dir = tempfile::tempdir().expect("临时目录");
    let paths = bill_core::DataPaths::new(dir.path().to_path_buf());
    paths.ensure_layout().expect("创建布局");
    // 运行期需要目录持续存在，测试进程结束时由系统回收。
    std::mem::forget(dir);
    std::sync::Arc::new(bill_api::Runtime::with_dataset(
        paths,
        bill_core::Dataset::default(),
    ))
}

/// GET /mcp 缺少会话与 SSE 请求头时应返回 4xx，而不是 5xx。
#[tokio::test]
async fn mcp_route_is_mounted() {
    let response = router(runtime())
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/mcp")
                .header(header::HOST, LOOPBACK_HOST)
                .body(Body::empty())
                .expect("请求构建失败"),
        )
        .await
        .expect("路由应可响应");

    assert!(
        response.status().is_client_error(),
        "GET /mcp 应返回 4xx，实际为 {}",
        response.status()
    );
}

/// 未注册的路径必须 404，避免 MCP 服务意外暴露其他路由。
#[tokio::test]
async fn unknown_route_returns_404() {
    let response = router(runtime())
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/not-exists")
                .header(header::HOST, LOOPBACK_HOST)
                .body(Body::empty())
                .expect("请求构建失败"),
        )
        .await
        .expect("路由应可响应");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

/// 非回环 Host 必须拒绝（防 DNS 重绑定，见技术方案 03 §13）。
#[tokio::test]
async fn foreign_host_is_rejected() {
    let response = router(runtime())
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/mcp")
                .header(header::HOST, "evil.example.com")
                .body(Body::empty())
                .expect("请求构建失败"),
        )
        .await
        .expect("路由应可响应");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

/// 请求体上限必须是 1MB（技术方案 03 §9.1），超出返回 413。
#[tokio::test]
async fn oversized_body_is_rejected() {
    let body = vec![b' '; 1024 * 1024 + 1];
    let response = router(runtime())
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/mcp")
                .header(header::HOST, LOOPBACK_HOST)
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ACCEPT, "application/json, text/event-stream")
                .body(Body::from(body))
                .expect("请求构建失败"),
        )
        .await
        .expect("路由应可响应");

    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("响应体读取失败");
    assert_eq!(
        status,
        StatusCode::PAYLOAD_TOO_LARGE,
        "超过 1MB 的请求体必须被拒绝，实际响应: {}",
        String::from_utf8_lossy(&body)
    );
}
