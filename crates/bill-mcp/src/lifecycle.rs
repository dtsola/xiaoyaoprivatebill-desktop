//! MCP 生命周期：回环监听、优雅停止、端口占用处理。

use std::fmt;
use std::sync::Mutex;
use std::time::Duration;

use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

use crate::{router_with_token, DEFAULT_PORT};

/// 停止服务的最长等待时间。
pub const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(3);

/// 启动失败原因。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum McpStartError {
    /// 端口被其他进程占用。
    PortInUse(u16),
    /// 其他启动错误。
    Server(String),
}

impl fmt::Display for McpStartError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PortInUse(port) => write!(formatter, "MCP 端口 {port} 已被占用"),
            Self::Server(message) => write!(formatter, "MCP 启动失败: {message}"),
        }
    }
}

impl std::error::Error for McpStartError {}

struct McpHandle {
    port: u16,
    shutdown: oneshot::Sender<()>,
    join: JoinHandle<()>,
}

/// MCP 服务管理器：固定端口、重复启动幂等、停止后释放端口。
pub struct McpManager {
    port: u16,
    running: Mutex<Option<McpHandle>>,
}

impl Default for McpManager {
    fn default() -> Self {
        Self::new(DEFAULT_PORT)
    }
}

impl McpManager {
    /// 以指定端口构造（测试用 `0` 表示由系统分配）。
    pub fn new(port: u16) -> Self {
        Self {
            port,
            running: Mutex::new(None),
        }
    }

    /// 配置的端口。
    pub fn port(&self) -> u16 {
        self.port
    }

    /// 实际监听端口；未运行时为 `None`。
    pub fn local_port(&self) -> Option<u16> {
        self.running
            .lock()
            .ok()
            .and_then(|guard| guard.as_ref().map(|handle| handle.port))
    }

    /// 是否正在运行。
    pub fn is_running(&self) -> bool {
        self.running
            .lock()
            .map(|guard| guard.is_some())
            .unwrap_or(false)
    }

    /// 启动服务；已在运行时直接返回成功。
    pub async fn start(&self, token: &str) -> Result<(), McpStartError> {
        if self.is_running() {
            return Ok(());
        }

        let listener = TcpListener::bind(("127.0.0.1", self.port))
            .await
            .map_err(|error| {
                if error.kind() == std::io::ErrorKind::AddrInUse {
                    McpStartError::PortInUse(self.port)
                } else {
                    McpStartError::Server(error.to_string())
                }
            })?;
        let port = listener
            .local_addr()
            .map_err(|error| McpStartError::Server(error.to_string()))?
            .port();

        let app = router_with_token(token);
        let (shutdown, receiver) = oneshot::channel();
        let join = tokio::spawn(async move {
            let _ = axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = receiver.await;
                })
                .await;
        });

        if let Ok(mut guard) = self.running.lock() {
            *guard = Some(McpHandle {
                port,
                shutdown,
                join,
            });
        }
        Ok(())
    }

    /// 优雅停止；未运行时为空操作。
    pub async fn stop(&self) {
        let handle = self.running.lock().ok().and_then(|mut guard| guard.take());
        if let Some(handle) = handle {
            let _ = handle.shutdown.send(());
            let _ = tokio::time::timeout(SHUTDOWN_TIMEOUT, handle.join).await;
        }
    }
}
