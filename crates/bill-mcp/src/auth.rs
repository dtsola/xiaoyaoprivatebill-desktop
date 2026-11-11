//! Bearer Token 与 Origin 校验。
//!
//! 规则见技术方案 03 §9.1：原生客户端可以不带 Origin；
//! 带 Origin 时只放行回环地址。Token 使用常量时间比较。

use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

/// 鉴权状态。
#[derive(Clone)]
pub struct Auth {
    token: Arc<str>,
}

impl Auth {
    /// 以 Token 构造。
    pub fn new(token: &str) -> Self {
        Self {
            token: Arc::from(token),
        }
    }
}

/// 中间件：先校验 Origin，再校验 Bearer Token。
pub async fn bearer_check(State(auth): State<Auth>, request: Request, next: Next) -> Response {
    if let Some(origin) = request.headers().get(header::ORIGIN) {
        let origin = origin.to_str().unwrap_or_default();
        if !origin_allowed(origin) {
            return error_response(StatusCode::FORBIDDEN, "Origin 不被允许");
        }
    }

    let token = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(bearer_value);
    let Some(token) = token else {
        return unauthorized();
    };
    if !constant_time_eq(token.as_bytes(), auth.token.as_bytes()) {
        return unauthorized();
    }

    next.run(request).await
}

/// 只放行回环 Origin：`http://127.0.0.1:<port>` / `http://localhost:<port>`。
pub fn origin_allowed(origin: &str) -> bool {
    ["http://127.0.0.1", "http://localhost"]
        .iter()
        .any(|prefix| {
            origin == *prefix
                || origin
                    .strip_prefix(prefix)
                    .is_some_and(|rest| rest.starts_with(':'))
        })
}

fn bearer_value(value: &HeaderValue) -> Option<&str> {
    let value = value.to_str().ok()?;
    let (scheme, token) = value.split_once(' ')?;
    if !scheme.eq_ignore_ascii_case("Bearer") {
        return None;
    }
    let token = token.trim();
    if token.is_empty() {
        None
    } else {
        Some(token)
    }
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut diff = 0u8;
    for (a, b) in left.iter().zip(right) {
        diff |= a ^ b;
    }
    diff == 0
}

fn unauthorized() -> Response {
    let mut response = error_response(StatusCode::UNAUTHORIZED, "缺少或无效的 MCP Token");
    response
        .headers_mut()
        .insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
    response
}

fn error_response(status: StatusCode, message: &str) -> Response {
    (status, Json(json!({ "error": message }))).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origin_whitelist_only_accepts_loopback() {
        assert!(origin_allowed("http://127.0.0.1:3000"));
        assert!(origin_allowed("http://localhost:5173"));
        assert!(origin_allowed("http://localhost"));
        assert!(!origin_allowed("https://evil.example"));
        assert!(!origin_allowed("http://localhost.evil.example"));
        assert!(!origin_allowed("http://127.0.0.2:3000"));
    }

    #[test]
    fn constant_time_compare_matches_exact_bytes() {
        assert!(constant_time_eq(b"abc", b"abc"));
        assert!(!constant_time_eq(b"abc", b"abd"));
        assert!(!constant_time_eq(b"abc", b"ab"));
    }
}
