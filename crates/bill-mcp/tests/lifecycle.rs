//! P4-3 契约测试：MCP 服务生命周期。

use std::net::{TcpListener, TcpStream};
use std::time::Duration;

use bill_mcp::lifecycle::{McpManager, McpStartError};

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

const TOKEN: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

#[tokio::test]
async fn start_accepts_connections_and_stop_releases_port() {
    let manager = McpManager::new(runtime(), 0);
    manager.start(TOKEN).await.expect("启动");

    assert!(manager.is_running());
    let port = manager.local_port().expect("实际端口");
    assert_ne!(port, 0);

    // 端口已可建立 TCP 连接。
    let mut connected = false;
    for _ in 0..20 {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            connected = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    assert!(connected, "启动后应能连接 127.0.0.1:{port}");

    manager.stop().await;
    assert!(!manager.is_running());

    // 关闭后端口应被释放。
    let mut released = false;
    for _ in 0..20 {
        if TcpListener::bind(("127.0.0.1", port)).is_ok() {
            released = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    assert!(released, "停止后端口 {port} 应可重新绑定");
}

#[tokio::test]
async fn occupied_port_returns_chinese_error_without_panic() {
    let occupied = TcpListener::bind(("127.0.0.1", 0)).expect("占位端口");
    let port = occupied.local_addr().expect("地址").port();

    let manager = McpManager::new(runtime(), port);
    let error = manager.start(TOKEN).await.expect_err("端口占用应失败");

    assert_eq!(error, McpStartError::PortInUse(port));
    assert_eq!(error.to_string(), format!("MCP 端口 {port} 已被占用"));
    assert!(!manager.is_running(), "失败后不应处于运行状态");
}

#[tokio::test]
async fn start_is_idempotent_and_stop_without_start_is_safe() {
    let manager = McpManager::new(runtime(), 0);
    manager.stop().await;

    manager.start(TOKEN).await.expect("首次启动");
    let port = manager.local_port().expect("端口");
    manager.start(TOKEN).await.expect("重复启动应幂等");

    assert_eq!(manager.local_port(), Some(port), "重复启动不应换端口");
    manager.stop().await;
}
