//! MCP 服务层。
//!
//! P0 阶段建立 axum + rmcp Streamable HTTP 传输骨架，只验证可编译、可挂载；
//! P4 再接入 14 个工具、Bearer Token 鉴权与审计。

use std::sync::Arc;

use axum::Router;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::{StreamableHttpServerConfig, StreamableHttpService};
use rmcp::ServerHandler;

/// MCP 监听端口。只允许绑定回环地址。
pub const DEFAULT_PORT: u16 = 17650;

/// MCP 路由前缀。
pub const ROUTE_PATH: &str = "/mcp";

/// 请求体上限 1MB（见技术方案 03 §9.1）。
pub const MAX_REQUEST_BODY_BYTES: usize = 1024 * 1024;

/// 最小 MCP 处理器。
///
/// P4 会用 14 个工具的实现替换这里的空处理器。
#[derive(Clone, Copy, Debug, Default)]
pub struct BillMcp;

impl ServerHandler for BillMcp {}

/// 构建 Streamable HTTP 服务。
///
/// 默认只接受 `127.0.0.1` / `localhost` 的 Host 头，请求体上限 1MB。
pub fn mcp_service() -> StreamableHttpService<BillMcp, LocalSessionManager> {
    let config = StreamableHttpServerConfig::default()
        .with_allowed_hosts(["127.0.0.1", "localhost"])
        .with_max_request_body_bytes(MAX_REQUEST_BODY_BYTES);

    StreamableHttpService::new(
        || Ok(BillMcp),
        Arc::new(LocalSessionManager::default()),
        config,
    )
}

/// 构建仅包含 MCP 路由的 axum Router。
pub fn router() -> Router {
    Router::new().nest_service(ROUTE_PATH, mcp_service())
}
