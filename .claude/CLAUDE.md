# 小遥账单桌面版 - 项目开发规范

> **文档版本**: v2.0
> **创建日期**: 2026-11-11
> **项目类型**: Tauri 桌面应用（Rust 后端 + Vue 前端冻结复用 + 内置 MCP）
> **当前状态**: 设计完成，编码未启动

---

## 目录

1. [项目概述](#1-项目概述)
2. [技术选型](#2-技术选型)
3. [代码规范](#3-代码规范)
4. [Git 规范](#4-git-规范)
5. [测试规范](#5-测试规范)
6. [部署规范](#6-部署规范)
7. [设计文档引用](#7-设计文档引用)
8. [开发约定](#8-开发约定)
9. [架构决策记录](#9-架构决策记录)

---

## 1. 项目概述

### 1.1 产品定位

小遥账单桌面版是一个**隐私优先**的个人账单分析桌面应用：

- 完全离线，不发起任何出站网络请求。
- 支持支付宝 CSV、微信 CSV/XLSX 账单解析。
- 多维度分析：年度、月度、分类、时间、消费洞察。
- 内置 MCP 服务，可被 Claude Code / Cursor 等 AI Agent 调用。
- 与 Web 版（`dtsola/xiaoyaoprivatebill`）同源：视觉与统计口径一致，但为独立产品，数据不互通。

### 1.2 项目结构

```
xiaoyaoprivatebill-desktop/
├─ Cargo.toml                     # workspace
├─ rust-toolchain.toml            # channel = "1.95"
├─ package.json                   # @tauri-apps/cli + 插件
├─ frontend/                      # 从 Web 版复制，冻结基线
│  ├─ PATCHES.md                  # 允许改动的 7 个文件清单
│  └─ src/...
├─ src-tauri/                     # Tauri 壳
│  ├─ tauri.conf.json
│  ├─ capabilities/default.json
│  └─ src/                        # main/lib/commands/tray/window/dragdrop/mcp_host
├─ crates/
│  ├─ bill-core/                  # 模型、错误、路径、设置、审计、时钟
│  ├─ bill-parsers/               # 支付宝/微信 CSV/XLSX、编码、文件名
│  ├─ bill-services/              # 数据集、导入、回收站、快照、统计、图表
│  ├─ bill-api/                   # 27 个 handler、路由分发、响应契约
│  └─ bill-mcp/                   # rmcp 服务、14 个工具、鉴权
├─ fixtures/                      # base / constructed / expected
├─ tools/golden/                  # 开发期对账脚本（不进入产物）
├─ scripts/check-frontend-baseline.mjs
├─ docs/                          # 正式文档 + 调研档案
└─ README.md / README_EN.md
```

依赖方向：

```
bill-core ← bill-parsers ← bill-services ← bill-api ← bill-mcp
                                              ↖
                                               src-tauri（壳）
```

禁止反向依赖与循环依赖；`bill-api` 不依赖 Tauri。

### 1.3 当前状态

| 项 | 状态 |
|---|---|
| 设计文档 | ✅ 100% |
| 代码 | ⬜ 0% |
| 测试与对账 | ⬜ 0% |
| 打包发布 | ⬜ 0% |

进度以 `docs/开发进度.md` 为准；任务以 `docs/04-开发任务清单.md` 为准。

### 1.4 与 Web 版的关系

- Web 版是**行为基准**，不是要修改的代码库。
- 前端从 Web 版复制并打 tag `web-baseline`，只允许改 7 个文件。
- 后端 100% Rust 重写；Python 仅用于开发期生成 golden，不进入产物。
- Web 版冻结，仅修复安全问题；新功能只进桌面版。

---

## 2. 技术选型

### 2.1 桌面与前端

| 技术 | 版本 | 用途 |
|---|---|---|
| Tauri | 2.x | 桌面壳、IPC、打包 |
| Rust | 1.95（MSVC） | 后端与业务逻辑 |
| Vue | 3.5.x | 前端（冻结复用） |
| Vite | 5.4.x | 前端构建 |
| Pinia | 2.1+ | 状态管理 |
| ECharts | 5.4+ | 数据可视化 |
| Node.js | 20 LTS / 24 | 前端构建 |

### 2.2 核心 crates

`serde` / `serde_json`、`tauri`、`tauri-plugin-single-instance`、`tauri-plugin-window-state`、`tauri-plugin-dialog`、`tauri-plugin-opener`、`axum`、`tokio`、`rmcp`（锁定 3.x）、`csv`、`calamine`、`encoding_rs`、`chrono`、`regex` / `fancy-regex`、`unicode-normalization`、`sha2`、`ulid`、`subtle`、`thiserror`、`tracing`、`schemars`。

### 2.3 明确不引入

| 技术 | 原因 |
|---|---|
| Electron / Node 后端 | 包体大、隐私叙事弱 |
| Python sidecar | 需打包解释器，两套运行时 |
| 数据库 / SQLite | 数据层为纯文件系统 |
| `polars` / `datafusion` | 数据量小，手写聚合更可控 |
| `reqwest` | 无出站请求 |
| updater / 遥测 / CDN | 与完全离线的隐私承诺冲突 |

### 2.4 环境要求

- Rust 1.95+（stable-x86_64-pc-windows-msvc）
- Node.js 20 LTS+、npm 10+
- Visual Studio 2022（Windows，含 VC Tools）
- WebView2（Windows 10/11）
- Python 3.10（仅 golden 对账）

---

## 3. 代码规范

### 3.1 命名规范

| 类型 | 规范 | 示例 |
|---|---|---|
| Rust 文件/模块 | snake_case | `data_store.rs` |
| Rust 类型 | PascalCase | `DatasetCell` |
| Rust 函数 | snake_case | `yearly_analysis` |
| 常量 | UPPER_SNAKE_CASE | `MAX_UPLOAD_SIZE` |
| Vue 组件 | PascalCase | `Settings.vue` |
| JS 函数/变量 | camelCase | `uploadPaths` |

### 3.2 Rust 规范

- 使用 `cargo fmt` 格式化。
- `cargo clippy --workspace -- -D warnings` 必须通过。
- 错误使用 `thiserror`；可预期错误用 `?` 或显式分支，禁止 `unwrap()`。
- 禁止跨 `.await` 持有 `std::sync::RwLock`。
- 锁顺序固定：`mutation → runtime → settings → mcp`，禁止反向。
- 解析与全量聚合放 `spawn_blocking`。
- 所有响应序列化前检查 NaN / Infinity，禁止输出。

```rust
/// 获取年度分析数据
///
/// # Errors
/// 当数据集不可用或参数无效时返回错误。
pub fn yearly_analysis(dataset: &Dataset, year: i32) -> Result<Value, ApiError> {
    // 实现...
}
```

### 3.3 Vue / 前端规范

- **只允许修改 7 个文件**：
  1. `frontend/src/api/client.js`
  2. `frontend/src/views/Settings.vue`
  3. `frontend/index.html`
  4. `frontend/src/router/index.js`
  5. `frontend/src/components/AppLayout.vue`
  6. `frontend/src/views/AboutAuthor.vue`
  7. `frontend/vite.config.js`
- 9 个页面（除 Settings）、3 个公共组件、4 个 Pinia store、`main.css` 保持冻结。
- 不新增前端依赖（`@tauri-apps/api`、dialog、opener 除外）。
- 新增样式写在对应 `<style scoped>` 内，不改全局主题。
- 视觉与 Web 版保持一致；不擅自重设计。

### 3.4 序列化与精度

- 金额使用 `f64`，与 Web 版一致；不使用 Decimal。
- 舍入规则见 `docs/03-技术方案文档.md` 第 7 节（18 项 pandas 兼容）。
- 空值按接口语义显式转换，禁止 `NaN` / `Infinity` 进入 JSON。

---

## 4. Git 规范

### 4.1 分支策略

```
main (稳定)
  └── dev (开发)
        └── feature/* (功能)
        └── fix/* (修复)
```

### 4.2 提交信息

采用 Conventional Commits：

```
<类型>(<范围>): <描述>

[可选的正文]
```

类型：`feat`、`fix`、`refactor`、`docs`、`style`、`test`、`chore`。

示例：

```bash
feat(api): 移植年度分析接口

实现 /yearly_analysis 的 Rust 版本，并通过 golden 对账：
- 年度收支汇总
- 月度趋势
- 分类占比

Co-Authored-By: Claude Code <noreply@anthropic.com>
```

### 4.3 提交边界

- 文档与代码分开提交。
- 前端冻结基线单独提交并打 tag `web-baseline`。
- golden 变更单独提交，说明原因并人工 review。
- 不提交 `target/`、`node_modules/`、`dist/`、`src-tauri/gen/schemas/`。

---

## 5. 测试规范

### 5.1 测试类型

| 类型 | 说明 | 目标 |
|---|---|---|
| 单元测试 | 解析器、pycompat、服务函数 | 核心逻辑覆盖 |
| 契约测试 | 27 个接口外层与错误形状 | 全部接口 |
| golden 对账 | Python 期望 JSON vs Rust 输出 | 27 接口 100%，容差 1e-6 |
| MCP 契约 | 14 工具、鉴权、写护栏、审计 | 全部工具 |
| 前端基线 | diff 仅 7 文件 | 100% |
| 手工验收 | 安装、托盘、导入、演示 | 每个里程碑 |

### 5.2 命令

```bash
cargo test --workspace
cargo test -p bill-services --test golden_contract
cargo test -p bill-parsers
cargo test -p bill-api --test envelopes
cargo test -p bill-mcp --test tools

python tools/golden/generate.py --out fixtures/expected --all
git diff --exit-code fixtures/expected

node scripts/check-frontend-baseline.mjs
```

### 5.3 对账规则

- 对象键集合相等，键顺序忽略；数组长度与顺序严格。
- 数字容差 `|a-b| <= 1e-6 * max(1,|a|,|b|)`。
- 禁止 NaN / Infinity。
- 不允许「改 golden 适配 Rust」；golden 变更必须说明是 Python 侧修正。

---

## 6. 部署规范

### 6.1 构建

```bash
npm --prefix frontend run build
npm run tauri build
# Windows：src-tauri/target/release/bundle/nsis/*.exe
```

macOS 通过 GitHub Actions 分架构构建：

- `macos-13` → `x86_64-apple-darwin`
- `macos-14` → `aarch64-apple-darwin`

### 6.2 版本

- 唯一真源：`src-tauri/tauri.conf.json` 的 `version`。
- 版本门槛：`0.1.0`（M3）→ `0.2.0`（M4）→ `0.3.0`（M5）→ `1.0.0`（M6）。
- `identifier` 固定 `com.dtsola.xiaoyaobill.desktop`，发布后不可更改。

### 6.3 数据目录

| 平台 | 路径 |
|---|---|
| Windows | `%APPDATA%\com.dtsola.xiaoyaobill.desktop` |
| macOS | `~/Library/Application Support/com.dtsola.xiaoyaobill.desktop` |

- 卸载不删除数据目录。
- `bills/` 是唯一真源；`trash/`、`snapshots/`、`audit.jsonl` 永不自动清理。

### 6.4 发布检查

- golden 27 接口全绿。
- 前端 diff 仅 7 文件。
- 断网启动零外部请求。
- MCP 默认关闭；开启后鉴权生效。
- Windows/macOS 干净环境安装可启动。
- README 包含 macOS 未签名打开方式与 MCP 隐私提示。

---

## 7. 设计文档引用

### 7.1 正式文档

| 文档 | 说明 |
|---|---|
| [00-mrd.md](../docs/00-mrd.md) | 市场需求：问题、用户、竞品、指标 |
| [01-prd.md](../docs/01-prd.md) | 产品需求：F1–F21、桌面行为、MCP、验收 |
| [02-原型+视觉.md](../docs/02-原型+视觉.md) | UI/UX 标准：页面基线、组件、桌面分组、导入交互、文案与验收 |
| [03-技术方案文档.md](../docs/03-技术方案文档.md) | 实现契约：架构、IPC、27 接口、存储、安全 |
| [04-开发任务清单.md](../docs/04-开发任务清单.md) | 61 个任务、工时、里程碑、质量门禁 |
| [05-开发排期.md](../docs/05-开发排期.md) | D1–D22、关键路径、风险 |
| [开发进度.md](../docs/开发进度.md) | 当前进度、Web 基线、技术债 |

### 7.2 调研档案（只读）

| 文档 | 说明 |
|---|---|
| [00-桌面端-可行性方案.md](../docs/桌面端-调研方案/00-桌面端-可行性方案.md) | 选型与否决理由、可行性 |
| [01-桌面端-prd.md](../docs/桌面端-调研方案/01-桌面端-prd.md) | 原始 PRD |
| [02-桌面端-原型+视觉.md](../docs/桌面端-调研方案/02-桌面端-原型+视觉.md) | 页面布局、设置页线框、托盘 |
| [03-桌面端-技术方案.md](../docs/桌面端-调研方案/03-桌面端-技术方案.md) | 原始技术契约 |
| [04-桌面端-开发实施步骤.md](../docs/桌面端-调研方案/04-桌面端-开发实施步骤.md) | 原始实施步骤 |

> 正式文档与档案冲突时，以正式文档为准；档案不修改。

### 7.3 模板

`docs/base/` 为精益开发模板（本地保留，已被 `.gitignore` 排除）。

---

## 8. 开发约定

### 8.1 语言偏好

**所有 AI 回复、文档编写、代码注释必须使用中文。**

- ✅ `// 获取年度分析数据`
- ❌ `// Get yearly analysis data`

### 8.2 功能完成检查清单

- [ ] 功能按 PRD 实现，验收标准可复现。
- [ ] 错误路径有中文文案，形状与 Python 一致。
- [ ] 单元测试 / golden 对账通过。
- [ ] 前端改动只在 7 个允许文件内。
- [ ] 审计覆盖新增写操作。
- [ ] 文档与任务状态已更新。
- [ ] 提交信息规范。

### 8.3 代码审查要点

**Rust**：

- API 响应形状、字段名、数组顺序是否与 Python 一致。
- 是否存在 `unwrap()`、跨 `.await` 锁、第二个 tokio Runtime。
- 写操作是否走 `mutation` 锁、是否写审计、失败是否回滚。
- 路径校验是否拒绝 `..`、NUL、分隔符。

**前端**：

- 是否超出 7 文件边界。
- 是否引入新依赖或改动全局样式。
- 断网时是否仍可完整使用。
- 外链是否走系统浏览器。

**MCP**：

- 默认是否关闭；监听是否仅 `127.0.0.1`。
- Token 是否常量时间比较；Origin 是否白名单。
- 写工具是否有护栏、审计、演示模式拒绝。

### 8.4 常用命令

```bash
# 开发
npm run tauri dev
npm --prefix frontend run dev

# 构建
npm --prefix frontend run build
npm run tauri build

# 质量
cargo fmt --check
cargo clippy --workspace -- -D warnings
cargo test --workspace

# 对账
python tools/golden/generate.py --out fixtures/expected --all
git diff --exit-code fixtures/expected

# 前端基线
node scripts/check-frontend-baseline.mjs
```

### 8.5 MCP 安全红线

- 只能监听 `127.0.0.1:17650`，**绝不监听 `0.0.0.0`**。
- 默认关闭；开启必须携带 Bearer Token。
- Token 不得写入日志、审计、错误信息。
- 写操作必须：参数校验 → 演示模式检查 → `mutation` 锁 → 审计 → 返回。
- 破坏性操作（`delete_bill`、`clear_data`）必须可恢复（回收站/快照）。
- 回收站、快照、审计日志永不自动清理。

### 8.6 数字口径

- 27 个接口 = 18 在用 + 9 兼容。
- 14 个 MCP 工具 = 10 读 + 4 写。
- 61 个任务；14–24 人日；日历 14–22 个工作日。
- 前端 7 个文件可改；9 个页面；演示数据 7,396 条。
- 不得写成「13 个工具」「前端零改动」「30/90 分钟自动清理」「Docker/Nginx 部署」。

---

## 9. 架构决策记录

### AD-20261111-001: 采用 Tauri v2 + Rust 重写后端

**日期**: 2026-11-11

**背景**: Web 版是 Flask + Vue + Docker，浏览器沙箱、启动门槛与无法常驻限制了产品形态。

**决策**: 采用 Tauri v2 作为桌面壳，后端 100% Rust 重写，前端 Vue 3 冻结复用。

**理由**: 安装包小（8–15MB 二进制）、系统 WebView、无 Python 运行时、单进程、隐私叙事强。

**影响**: 需要 3–5 人日复刻 pandas 统计口径；27 个接口需逐字段对账。

### AD-20261111-002: 前端冻结复用，只改 7 个文件

**日期**: 2026-11-11

**背景**: 9 个页面与视觉已经稳定，重写成本高且容易引入回归。

**决策**: 复制 Web 版 `frontend/` 作为冻结基线，只允许改 7 个文件。

**影响**: 联调返工可控；用 `check-frontend-baseline.mjs` 做 CI 门禁。

### AD-20261111-003: 内置 MCP 服务

**日期**: 2026-11-11

**背景**: Agent 是新的查询入口；单独做一套 API 会产生第二份缓存与不一致。

**决策**: 在应用内启动 Streamable HTTP MCP 服务，复用同一套 service 层；14 个工具；默认关闭。

**影响**: 增加 MCP 生命周期、鉴权、审计与写护栏的复杂度；换来确定性与隐私可控的 Agent 接入。

### AD-20261111-004: 保留 Web 版已知缺陷 D1–D7

**日期**: 2026-11-11

**背景**: 桌面版要求统计口径与 Web 版逐字段一致。

**决策**: 本期 100% 复刻，包括支付宝退款判定、重复导入不去重等缺陷。

**影响**: 用户可能看到与真实退款不符的数字；修复必须作为独立版本行为变更，并同步更新 golden 与用户说明。

---

**文档维护**: 开发团队
**最后更新**: 2026-11-11
