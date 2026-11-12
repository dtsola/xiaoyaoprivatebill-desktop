# ADR-0001：Tauri / rmcp / axum 编译 Spike 结论

- **日期**：2026-11-11
- **状态**：已采纳
- **关联任务**：`docs/04-开发任务清单.md` P0-5
- **关联文档**：`docs/03-技术方案文档.md` §9、§10、§14

---

## 1. 背景

P4 的 MCP 服务与 P5 的桌面行为都依赖一批尚未在本仓库验证过的 API。若等到 P4/P5 才发现 API 形状不符，会同时阻塞 GUI 与 MCP 两条线。因此 P0 用**编译 spike**（`cargo check` + 最小可运行测试）先确认每个 API 的真实签名与替代写法。

Spike 只验证「能编译、能构造、行为符合预期」，不实现业务逻辑。

## 2. 结论总览

| # | Spike 项 | 结论 | 证据 |
|---|---|---|---|
| 1 | Tauri 托盘 `tray-icon` | ✅ 编译通过 | `src-tauri/src/tray.rs`；`cargo check --workspace` |
| 2 | 单实例插件 | ✅ 编译通过（必须第一个注册） | `src-tauri/src/lib.rs` |
| 3 | 窗口状态插件 | ✅ 编译通过 | `src-tauri/src/lib.rs` |
| 4 | dialog 插件 | ✅ 编译通过 + capability 就位 | `capabilities/default.json`、`dialog:allow-open` |
| 5 | opener 插件 | ✅ 编译通过 + capability 就位 | `capabilities/default.json`、`opener:allow-open-url` |
| 6 | 拖拽导入（窗口级） | ✅ 配置项存在 | `tauri.conf.json` `dragDropEnabled: true` |
| 7 | rmcp 3.9 Streamable HTTP | ✅ 编译通过，签名已确认 | `crates/bill-mcp/src/lib.rs` |
| 8 | axum 0.8 挂载 rmcp 服务 | ✅ 编译通过 + 4 个契约测试 | `crates/bill-mcp/tests/router.rs` |
| 9 | NSIS / dmg 打包 | ⚠️ 未验证，推迟到 P6 | 见 §4.3 |

## 3. 关键 API 结论

### 3.1 rmcp 3.9

**依赖写法**（`crates/bill-mcp/Cargo.toml`）：

```toml
rmcp = { version = "3.9", default-features = false, features = [
    "server",
    "macros",
    "transport-streamable-http-server",
] }
```

- `transport-streamable-http-server` 已隐含 `server-side-http`、`transport-worker`、`transport-streamable-http-server-session`，**不需要**额外开启 `server-side-http`。
- 不使用 `local` feature，否则 `StreamableHttpService` 的顶层再导出会被 `cfg(not(feature = "local"))` 关掉。

**服务构造签名（已确认）**：

```rust
StreamableHttpService::new(
    service_factory: impl Fn() -> Result<S, std::io::Error> + Send + Sync + 'static,
    session_manager: Arc<LocalSessionManager>,
    config: StreamableHttpServerConfig,
) -> StreamableHttpService<S, LocalSessionManager>
```

其中 `S: ServerHandler + Send + 'static`，`ServerHandler` 的所有方法都有默认实现，P4 之前可用空实现：

```rust
#[derive(Clone, Copy, Debug, Default)]
pub struct BillMcp;

impl ServerHandler for BillMcp {}
```

**会话管理器路径**：

```rust
rmcp::transport::streamable_http_server::session::local::LocalSessionManager
```

**配置默认值（实测）**：

| 项 | 默认值 | 本项目取值 |
|---|---|---|
| `allowed_hosts` | `localhost`、`127.0.0.1`、`::1` | 收窄为 `127.0.0.1`、`localhost` |
| `max_request_body_bytes` | 4MB | **显式改为 1MB**（技术方案 03 §9.1） |
| `legacy_session_mode` | `true` | 保持默认 |
| `sse_keep_alive` | 15s | 保持默认 |

> **重要**：`max_request_body_bytes` 默认是 4MB，与文档写的 1MB 不一致，必须显式调用 `with_max_request_body_bytes(1024 * 1024)`，否则契约不成立。

### 3.2 axum 0.8

```rust
Router::new().nest_service("/mcp", mcp_service())
```

- rmcp 的 `StreamableHttpService` 实现了 `tower_service::Service<http::Request<Body>>`，`Error = Infallible`，可直接 `nest_service`，不需要手写适配层。
- `StreamableHttpService: Clone`，满足 axum `nest_service` 的 `Clone` 约束。

### 3.3 传输层行为（已被测试锁定）

`crates/bill-mcp/tests/router.rs` 覆盖 4 条：

| 场景 | 期望 | 实测 |
|---|---|---|
| `GET /mcp` 无会话、无 SSE 头 | 4xx | 400 |
| `GET /not-exists` | 404 | 404 |
| `Host: evil.example.com` | 拒绝 | **403** |
| `POST /mcp` 请求体 1MB+1 | 413 | 413 |

补充约束（来自 rmcp 源码，P4 写客户端说明时要用）：

- `Host` 头缺失 → `400 Bad Request: missing Host header`。
- `Accept` 必须同时包含 `application/json` 与 `text/event-stream`，否则 `406`。
- `Content-Type` 必须是 `application/json`（允许带 `; charset=`），否则 `415`。

### 3.4 Tauri 插件

| 插件 | 初始化写法 | 注意 |
|---|---|---|
| 单实例 | `tauri_plugin_single_instance::init(...)` | **必须是第一个注册的插件**，否则第二实例可能先创建窗口 |
| 窗口状态 | `tauri_plugin_window_state::Builder::default().build()` | 与 `visible: false` 配合：setup 里显示窗口后再恢复状态 |
| dialog | `tauri_plugin_dialog::init()` | 前端只允许 `dialog:allow-open` |
| opener | `tauri_plugin_opener::init()` | 前端只允许 `opener:allow-open-url`，外链走系统浏览器 |
| 托盘 | `TrayIconBuilder::with_id("main")` + `Menu` | 需要 feature `tray-icon`、`image-png`；`icon_as_template(true)` 仅 macOS 有意义 |

## 4. 风险与未决项

### 4.1 已消除

- rmcp 顶层导出受 `local` feature 影响的坑：已通过不启用 `local` 规避。
- 请求体上限默认值不符：已显式配置。
- axum / rmcp 的 `http` crate 版本冲突：两者均使用 `http 1.x`，无冲突。

### 4.2 遗留（P4 处理）

- Bearer Token 鉴权未在 spike 中实现。P4 需在 `Router` 外层加中间件，保持常量时间比较（`subtle`）。
- `ServerHandler` 只做了空实现，14 个工具的 `#[tool_router]` / `#[tool_handler]` 宏未验证；P4-2 第一个工具落地时补 spike。
- SSE 长连接与托盘退出时的连接释放未验证。

### 4.3 打包（P6 处理）

`cargo check` 不覆盖打包，以下风险保留到 P6：

| 风险 | 说明 |
|---|---|
| 中文 `productName` | NSIS 对中文产品名的安装路径/快捷方式处理需实机验证 |
| `webviewInstallMode: skip` | 不内置 WebView2，Win10 干净环境需用户自行安装 |
| macOS 未签名 | 需在 README 给出右键打开方式；CI 双架构构建未跑通 |

## 5. 影响

- P4 可直接按本文档的签名实现 MCP 服务，无需再翻 rmcp 源码。
- P5 的插件注册顺序被固定：**单实例 → 窗口状态 → dialog → opener**。
- 技术方案 03 §9.1 的 1MB 上限从「文档约定」变为「可执行断言」。

## 6. 变更历史

| 日期 | 版本 | 说明 |
|---|---|---|
| 2026-11-11 | v1.0 | 首次记录 P0-5 spike 结论 |
