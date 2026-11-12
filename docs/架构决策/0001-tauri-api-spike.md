# 架构决策 - Tauri / rmcp / axum 编译 Spike 结论

**决策日期**：2026年11月11日

## 背景

P4 的 MCP 服务与 P5 的桌面行为依赖一批尚未在本仓库验证过的 API。若等到 P4/P5 才发现 API 形状不符，会同时阻塞 GUI 与 MCP 两条线。因此 P0-5 用编译 spike（`cargo check` + 最小可运行测试）先确认每个 API 的真实签名与替代写法，只验证「能编译、能构造、行为符合预期」，不实现业务逻辑。

## 解决方案

先对 9 个关键点做编译 spike，结论如下：

| Spike 项 | 结论 | 证据 |
|---|---|---|
| Tauri 托盘 `tray-icon` | 编译通过 | `src-tauri/src/tray.rs` |
| 单实例插件 | 编译通过，必须第一个注册 | `src-tauri/src/lib.rs` |
| 窗口状态插件 | 编译通过 | `src-tauri/src/lib.rs` |
| dialog 插件 | 编译通过，capability 就位 | `dialog:allow-open` |
| opener 插件 | 编译通过，capability 就位 | `opener:default`（含默认 URL scope） |
| 拖拽导入 | 配置项存在 | `dragDropEnabled: true` |
| rmcp 3.9 Streamable HTTP | 编译通过，签名已确认 | `crates/bill-mcp/src/lib.rs` |
| axum 0.8 挂载 rmcp | 编译通过 + 4 个契约测试 | `crates/bill-mcp/tests/router.rs` |
| NSIS / dmg 打包 | 未验证，推迟到 P6 | 见下文「影响」 |

关键结论：

- **rmcp 依赖写法**：`default-features = false`，只开 `server`、`macros`、`transport-streamable-http-server`；不启用 `local`，否则 `StreamableHttpService` 顶层再导出会被关掉。
- **服务构造**：`StreamableHttpService::new(factory, LocalSessionManager, config)`；`ServerHandler` 全部方法有默认实现，P4 前可用空实现。
- **请求体上限**：`max_request_body_bytes` 默认 4MB，必须显式 `with_max_request_body_bytes(1MB)`，与 03 §9.1 的契约一致。
- **axum 挂载**：`Router::new().nest_service("/mcp", mcp_service())`，无需手写适配层。
- **传输层约束**（已被 `tests/router.rs` 锁定）：无会话 GET 返回 400；未知路径 404；`Host: evil.example.com` 返回 403；1MB+1 请求体返回 413；`Accept` 必须同时含 JSON 与 SSE，否则 406。
- **Tauri 插件注册顺序固定**：单实例 → 窗口状态 → dialog → opener。
- **外链权限**：只写 `opener:allow-open-url` 会因缺少 URL scope 被拒绝；实际使用 `opener:default`（含 `allow-default-urls`，放行 http/https/mailto/tel）。

## 影响

- P4 可直接按上述签名实现 MCP 服务，无需再翻 rmcp 源码；P5 的插件注册顺序被固定。
- 1MB 请求体上限从「文档约定」变成「可执行断言」。
- 遗留项：Bearer Token 鉴权、14 个工具的宏、SSE 长连接释放由 P4 补齐；中文 `productName`、`webviewInstallMode: skip`、macOS 未签名与双架构构建由 P6 实机验证。

---

**决策编号**：AD-20261111-005
