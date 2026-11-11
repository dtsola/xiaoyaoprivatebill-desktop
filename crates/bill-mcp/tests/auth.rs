//! P4-2 契约测试：Bearer 与 Origin 校验。

use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use bill_mcp::router_with_token;
use serde_json::json;
use tower::ServiceExt;

const LOOPBACK_HOST: &str = "127.0.0.1:17650";
const TOKEN: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn mcp_request(token: Option<&str>, origin: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder()
        .method(Method::POST)
        .uri("/mcp")
        .header(header::HOST, LOOPBACK_HOST)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::ACCEPT, "application/json, text/event-stream");
    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    if let Some(origin) = origin {
        builder = builder.header(header::ORIGIN, origin);
    }
    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "0"}
        }
    });
    builder.body(Body::from(body.to_string())).expect("请求")
}

#[tokio::test]
async fn missing_authorization_is_rejected() {
    let response = router_with_token(TOKEN)
        .oneshot(mcp_request(None, None))
        .await
        .expect("响应");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn wrong_token_is_rejected() {
    let response = router_with_token(TOKEN)
        .oneshot(mcp_request(Some("deadbeef"), None))
        .await
        .expect("响应");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn foreign_origin_is_forbidden_even_with_valid_token() {
    let response = router_with_token(TOKEN)
        .oneshot(mcp_request(Some(TOKEN), Some("https://evil.example")))
        .await
        .expect("响应");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn loopback_origin_and_valid_token_pass() {
    let response = router_with_token(TOKEN)
        .oneshot(mcp_request(Some(TOKEN), Some("http://127.0.0.1:3000")))
        .await
        .expect("响应");

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn native_client_without_origin_passes() {
    let response = router_with_token(TOKEN)
        .oneshot(mcp_request(Some(TOKEN), None))
        .await
        .expect("响应");

    assert_eq!(response.status(), StatusCode::OK);
}
