# 小遥账单桌面版

> 隐私优先的个人账单分析桌面应用：完全离线、数据本地持久化、可被 AI Agent 通过 MCP 调用。

[![Tauri](https://img.shields.io/badge/Tauri-2.x-24C8DB.svg)](https://tauri.app/)
[![Rust](https://img.shields.io/badge/Rust-1.95+-orange.svg)](https://www.rust-lang.org/)
[![Vue](https://img.shields.io/badge/Vue-3.5+-brightgreen.svg)](https://vuejs.org/)
[![Windows](https://img.shields.io/badge/Windows-10%2F11%20x64-blue.svg)](https://www.microsoft.com/windows)
[![macOS](https://img.shields.io/badge/macOS-11%2B-lightgrey.svg)](https://www.apple.com/macos/)
[![MCP](https://img.shields.io/badge/MCP-14%20tools-purple.svg)](https://modelcontextprotocol.io/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

[English](README_EN.md) | 简体中文

<p align="center">
  <img src="docs/产品文档/产品截图/宣传海报图.png" alt="小遥账单桌面版宣传海报">
</p>

> **当前状态**：设计完成，编码尚未启动（v0.1.0 开发中），安装包尚未发布。现阶段请使用「[从源码构建](#8-开发构建)」。

---

## 1. 简介

<p align="center">
  <img src="docs/产品文档/logos/logo.png" alt="小遥账单桌面版" width="200">
</p>

小遥账单桌面版是一个**隐私优先**的个人账单分析工具，支持支付宝和微信账单的自动解析与多维度可视化分析，并内置 MCP 服务，让 Claude Code、Cursor 等 AI Agent 可以直接查询你的账单。

**核心特点**：

- 🔒 **完全离线** - 不发起任何出站网络请求，无遥测、无 CDN、无自动更新
- 💾 **本地持久化** - 账单保存在本机应用数据目录，随时手动清理
- 📊 **多维度分析** - 年度、月度、分类、时间、消费洞察
- 📁 **多格式支持** - 支付宝 CSV、微信 CSV/XLSX
- 🤖 **Agent 可调用** - 内置 14 个 MCP 工具，自然语言查账
- 🖥️ **桌面原生** - 系统托盘、单实例、窗口状态记忆、原生文件导入
- 🗑️ **可恢复删除** - 回收站、快照、审计日志，降低误操作风险
- 🧬 **与 Web 版同源** - 9 个页面视觉与统计口径完全一致，后端 Rust 重写

---

## 2. 功能预览

> 以下界面与 Web 版保持一致；桌面版新增的设置分组、托盘与 MCP 配置截图将在发布前补齐。

### 上传账单 - 一键导入

<table>
<tr>
<td width="50%"><img src="docs/产品文档/产品截图/08-上传账单.png" alt="上传账单"></td>
<td width="50%">
<ul>
<li>📤 原生文件对话框，支持多选与全窗口拖拽</li>
<li>📋 自动识别支付宝/微信账单格式</li>
<li>⚡ 本地解析，不上传任何服务器</li>
</ul>
</td>
</tr>
</table>

### 首页 - 功能入口

<table>
<tr>
<td width="50%"><img src="docs/产品文档/产品截图/01-首页.png" alt="首页"></td>
<td width="50%">
<ul>
<li>🎯 支持支付宝和微信账单的一站式分析</li>
<li>📊 4 大能力卡片：多维度分析、隐私保护、趋势追踪、智能搜索</li>
<li>🚀 导入账单或进入演示模式</li>
</ul>
</td>
</tr>
</table>

### 年度总览 - 一目了然

<table>
<tr>
<td width="50%"><img src="docs/产品文档/产品截图/02-年度总览.png" alt="年度总览"></td>
<td width="50%">
<ul>
<li>📊 年度收支总览</li>
<li>📈 月度趋势与分类占比</li>
<li>📖 年度故事回顾</li>
</ul>
</td>
</tr>
</table>

### 月度分析 - 精细对比

<table>
<tr>
<td width="50%"><img src="docs/产品文档/产品截图/03-月度分析.png" alt="月度分析"></td>
<td width="50%">
<ul>
<li>📅 月度收支与环比</li>
<li>🗓️ 每日消费日历</li>
<li>📉 月度对比与分类明细</li>
</ul>
</td>
</tr>
</table>

### 分类分析 - 花费明细

<table>
<tr>
<td width="50%"><img src="docs/产品文档/产品截图/04-分类分析.png" alt="分类分析"></td>
<td width="50%">
<ul>
<li>🏷️ 分类占比与排行</li>
<li>📊 分类趋势与时段分布</li>
<li>📝 分类明细列表</li>
</ul>
</td>
</tr>
</table>

### 时间分析 - 消费规律

<table>
<tr>
<td width="50%"><img src="docs/产品文档/产品截图/05-时间分析.png" alt="时间分析"></td>
<td width="50%">
<ul>
<li>⏰ 24 小时消费分布</li>
<li>🌡️ 日历热力图</li>
<li>📈 工作日/周末对比</li>
</ul>
</td>
</tr>
</table>

### 消费洞察 - 智能分析

<table>
<tr>
<td width="50%"><img src="docs/产品文档/产品截图/06-消费洞察-01.png" alt="消费洞察-01"></td>
<td width="50%"><img src="docs/产品文档/产品截图/06-消费洞察-02.png" alt="消费洞察-02"></td>
</tr>
<tr>
<td width="50%"><img src="docs/产品文档/产品截图/06-消费洞察-03.png" alt="消费洞察-03"></td>
<td width="50%">
<ul>
<li>💡 消费习惯与画像</li>
<li>🔍 异常消费与拿铁因子</li>
<li>📊 深夜消费、订阅、恩格尔系数</li>
</ul>
</td>
</tr>
</table>

### 交易记录 - 明细查询

<table>
<tr>
<td width="50%"><img src="docs/产品文档/产品截图/07-交易记录.png" alt="交易记录"></td>
<td width="50%">
<ul>
<li>📋 完整交易明细</li>
<li>🔍 搜索与多维筛选</li>
<li>📄 分页浏览</li>
</ul>
</td>
</tr>
</table>

<!-- TODO(P6): 补 09-桌面.png（设置页「桌面」分组） -->
<!-- TODO(P6): 补 10-系统托盘.png（托盘菜单） -->
<!-- TODO(P6): 补 11-MCP配置.png（MCP 开关与配置复制） -->

---

## 3. 安装

### 3.1 Windows 10 / 11（x64）

1. 从 [GitHub Releases](https://github.com/dtsola/xiaoyaoprivatebill-desktop/releases) 下载 `*-setup.exe`。
2. 双击安装，按向导完成；安装范围为当前用户，无需管理员权限。
3. 从开始菜单启动「小遥账单桌面版」。

> Windows 10 需要已安装 WebView2 运行时（Windows 11 自带）。若启动失败，请先安装 [WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/)。

### 3.2 macOS 11+（Intel / Apple Silicon）

1. 从 [GitHub Releases](https://github.com/dtsola/xiaoyaoprivatebill-desktop/releases) 下载对应架构的 `.dmg`：
   - Intel：`*_x64.dmg`
   - Apple Silicon：`*_aarch64.dmg`
2. 打开 dmg，将应用拖入「应用程序」。
3. **首次打开**：右键点击应用 →「打开」；或到「系统设置 → 隐私与安全性」点击「仍要打开」。
4. macOS 15 如仍被拦截，可在终端执行：

```bash
xattr -dr com.apple.quarantine "/Applications/小遥账单桌面版.app"
```

> 当前版本**未签名、未公证**，因此会有上述提示。后续版本将补充签名与公证。

### 3.3 系统要求

| 项 | 要求 |
|---|---|
| Windows | Windows 10 / 11 x64；已安装 WebView2 |
| macOS | macOS 11+；Intel 或 Apple Silicon |
| 磁盘 | 安装包约 20MB；数据目录随账单增长 |
| 网络 | **不需要**；断网可完整使用 |

### 3.4 数据目录与卸载

| 平台 | 数据目录 |
|---|---|
| Windows | `%APPDATA%\com.dtsola.xiaoyaobill.desktop` |
| macOS | `~/Library/Application Support/com.dtsola.xiaoyaobill.desktop` |

- 卸载应用**不会**删除数据目录；如需彻底清除，请手动删除上述目录。
- 备份时复制整个数据目录即可；`bills/` 是唯一真源。

---

## 4. 快速开始

### 4.1 获取账单文件

**支付宝账单**：

1. 打开支付宝 App → 账单。
2. 点击右上角「常见问题」→ 导出账单。
3. 选择「个人账单」→ 时间范围 → 选择 CSV 格式。
4. 输入邮箱，等待接收账单文件。

**微信账单**：

1. 打开微信 → 我 → 服务 → 钱包 → 右上角「账单」。
2. 点击「常见问题」→ 下载账单。
3. 选择时间范围 → 输入邮箱 → 选择 CSV 或 XLSX 格式。

### 4.2 导入并分析

1. 打开应用，点击「上传账单」或进入「设置」。
2. 在支付宝/微信上传区点击选择文件，或直接把账单拖入窗口。
3. 等待解析完成；成功后可查看年度、月度、分类、时间、洞察与明细。
4. 如需先体验，点击「体验演示数据」加载内置 7,396 条示例数据。

### 4.3 演示模式

- 演示模式下可浏览全部 9 个页面。
- 演示模式下**禁止**导入、删除、清空，以及所有 MCP 写操作。
- 退出演示后回到真实数据。

---

## 5. 使用 MCP

### 5.1 开启服务

1. 打开「设置 → 桌面 → MCP 服务」。
2. 打开开关；状态显示「运行中」。
3. 端点固定为 `http://127.0.0.1:17650/mcp`，仅监听本机回环地址。
4. 点击「复制 MCP 配置」，粘贴到你的 MCP 客户端。

> MCP 默认关闭。开启后，本机程序可读取你的全部账单；数据是否离开本机取决于调用方（Agent）。

### 5.2 客户端配置

```json
{
  "mcpServers": {
    "xiaoyao-bill": {
      "type": "http",
      "url": "http://127.0.0.1:17650/mcp",
      "headers": {
        "Authorization": "Bearer <TOKEN>"
      }
    }
  }
}
```

Claude Code 也可以直接添加：

```bash
claude mcp add --transport http xiaoyao-bill http://127.0.0.1:17650/mcp \
  --header "Authorization: Bearer <TOKEN>"
```

### 5.3 14 个工具

| 类型 | 工具 |
|---|---|
| 读（10） | `list_bills`、`get_overview`、`get_summary`、`get_yearly_analysis`、`get_monthly_analysis`、`get_category_analysis`、`search_transactions`、`get_time_analysis`、`get_insights`、`get_audit_log` |
| 写（4） | `import_bill`、`delete_bill`、`restore_bill`、`clear_data` |

写操作有护栏：`import_bill` 仅接受绝对路径且 ≤16MB；`delete_bill` 软删除到回收站；`clear_data` 必须先快照；所有写操作写入审计日志。

### 5.4 安全提示

- Token 为 32 字节随机数，可在设置页重新生成；重新生成后旧连接失效。
- 服务只监听 `127.0.0.1`，不监听局域网或公网。
- 回收站、快照、审计日志**永不自动清理**，请在设置页手动清理。
- 若不需要 Agent 访问，请保持 MCP 关闭。

---

## 6. 隐私说明

| 项 | 说明 |
|---|---|
| 网络请求 | 应用本身不发起任何出站请求；无遥测、无崩溃上报、无自动更新 |
| 字体与图标 | Font Awesome 本地化，不加载 CDN |
| 数据存储 | 仅保存在本机应用数据目录；不上传任何服务器 |
| 数据清理 | 删除进入回收站；清空前自动快照；审计日志记录写操作；均需手动清理 |
| 外链 | 关于作者等外链交给系统浏览器打开 |
| MCP | 默认关闭；仅回环；Bearer Token；写操作审计；是否让数据离开本机由调用方决定 |
| 卸载 | 不删除数据目录；需要时手动删除 |

---

## 7. 从 Web 版迁移

- 桌面版是**独立产品**，不读取、不迁移 Web 版的数据目录。
- 两个版本的视觉与统计口径对齐，包括已知缺陷（如支付宝退款判定）。
- 迁移方式：在桌面版重新导入原始账单文件即可。
- Web 版已冻结，仅修复安全问题；新功能只进桌面版。
- 同源 Web 版：[dtsola/xiaoyaoprivatebill](https://github.com/dtsola/xiaoyaoprivatebill)。

---

## 8. 开发构建

### 8.1 前置要求

| 工具 | 版本 |
|---|---|
| Rust | 1.95+（MSVC 工具链，Windows） |
| Node.js | 20 LTS 或 24 |
| npm | 10+ |
| Visual Studio Build Tools | 2022（Windows，含 VC Tools） |
| WebView2 | Windows 10/11 |
| Python | 3.10（仅 golden 对账，不进入产物） |

### 8.2 本地开发

```bash
# 1. 克隆仓库
git clone https://github.com/dtsola/xiaoyaoprivatebill-desktop.git
cd xiaoyaoprivatebill-desktop

# 2. 安装依赖
npm install
npm --prefix frontend install

# 3. 启动开发模式（Vite + Tauri）
npm run tauri dev

# 4. 仅前端开发
npm --prefix frontend run dev
```

### 8.3 构建安装包

```bash
# 构建前端
npm --prefix frontend run build

# 构建桌面安装包（Windows NSIS）
npm run tauri build
# 产物：src-tauri/target/release/bundle/nsis/*.exe
```

macOS 通过 GitHub Actions 分架构构建，产物为 `x86_64.dmg` 与 `aarch64.dmg`，工作流文件为 `.github/workflows/macos-build.yml`。

### 8.4 质量与对账

```bash
cargo fmt --check
cargo clippy --workspace -- -D warnings
cargo test --workspace

# golden 对账（需 Python 3.10 + 锁定依赖）
python tools/golden/generate.py
git diff --exit-code fixtures/expected

# 前端冻结基线校验（只允许 7 个文件 diff）
node scripts/check-frontend-baseline.mjs
```

---

## 9. 文档

| 文档 | 说明 |
|---|---|
| [市场需求文档（MRD）](docs/00-mrd.md) | 市场问题、目标用户、竞品、成功指标 |
| [产品需求文档（PRD）](docs/01-prd.md) | 功能清单、桌面行为、MCP、页面、验收 |
| [原型与视觉标准](docs/02-原型+视觉.md) | 页面基线、组件、桌面分组、导入交互、文案与视觉验收 |
| [技术方案文档](docs/03-技术方案文档.md) | Tauri + Rust 架构、27 接口、存储、安全、打包 |
| [开发任务清单](docs/04-开发任务清单.md) | 61 个任务、工时、里程碑、质量门禁 |
| [开发排期](docs/05-开发排期.md) | D1–D22、关键路径、风险 |
| [开发进度](docs/开发进度.md) | 当前进度、Web 基线、技术债 |
| [桌面端调研方案](docs/桌面端-调研方案/) | 可行性、PRD、原型视觉、技术方案、实施步骤（只读档案） |

---

## 10. 常见问题

### Q：Windows 10 启动失败，提示缺少 WebView2？

A：安装 [WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/) 后重试。Windows 11 通常自带。

### Q：macOS 提示「无法验证开发者」？

A：右键点击应用 →「打开」；或到「系统设置 → 隐私与安全性」点击「仍要打开」。也可执行上文 `xattr` 命令。

### Q：MCP 开启失败，提示端口被占用？

A：端口 `17650` 被其他程序占用。请关闭占用程序后重试，或暂时保持 MCP 关闭；GUI 功能不受影响。

### Q：导入失败怎么办？

A：确认：
- 支付宝为 `.csv`，编码 GBK 或 UTF-8；
- 微信为 `.csv` 或 `.xlsx`；
- 单文件不超过 16MB；
- **文件名使用英文/数字**（中文文件名可能被清洗后无法识别，这是为与 Web 版保持一致而保留的行为）。

### Q：重复导入会怎样？

A：不会按内容去重；同名文件会追加 `_1`、`_2`，重复交易会重复计入统计。这是与 Web 版一致的已知行为。

### Q：数据会上传到服务器吗？

A：不会。应用本身不发起任何出站网络请求。只有你主动开启 MCP 并让 Agent 调用时，数据才可能由 Agent 带出本机。

### Q：如何备份或迁移数据？

A：复制整个数据目录（见 [3.4](#34-数据目录与卸载)）。`bills/` 是账单真源，`trash/`、`snapshots/`、`audit.jsonl` 分别是回收站、快照与审计日志。

### Q：支持 Linux 或手机吗？

A：不支持。本期仅支持 Windows 10/11 x64 与 macOS 11+。

---

## 11. 致谢

- 本项目的前端与产品设计同源于 [dtsola/xiaoyaoprivatebill](https://github.com/dtsola/xiaoyaoprivatebill)（Web 版，已冻结）。
- Web 版最初重构自 [alipay_record_analysis](https://github.com/Hessel2333/alipay_record_analysis)，感谢原作者 **Hessel2333**。
- 桌面端基于 [Tauri](https://tauri.app/)、[Rust](https://www.rust-lang.org/)、[Vue 3](https://vuejs.org/)、[ECharts](https://echarts.apache.org/)、[rmcp](https://github.com/modelcontextprotocol/rust-sdk)。

---

## 12. 贡献指南

欢迎贡献代码、报告问题或提出建议。

### 开发流程

1. Fork 本仓库。
2. 创建功能分支：`git checkout -b feature/AmazingFeature`。
3. 提交更改：`git commit -m 'feat: 添加某个功能'`。
4. 推送分支：`git push origin feature/AmazingFeature`。
5. 创建 Pull Request。

### 代码规范

- Rust 遵循 `cargo fmt` 与 `cargo clippy -- -D warnings`。
- 前端只允许修改 7 个基线文件，其余保持冻结。
- 提交信息遵循 [Conventional Commits](https://www.conventionalcommits.org/)。
- 详细规范见 [.claude/CLAUDE.md](.claude/CLAUDE.md)。

### 提交前检查

```bash
cargo fmt --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
node scripts/check-frontend-baseline.mjs
```

---

## 13. 许可证与联系

本项目采用 [MIT 许可证](LICENSE)。

<p align="center">
  <img src="docs/产品文档/产品截图/作者头像.jpg" alt="dtsola" width="100" height="100" style="border-radius: 50%;">
</p>

<p align="center">
  <b>dtsola</b> — IT 解决方案架构师 | 一人公司实践者
</p>

<p align="center">
  🌐 <a href="https://www.dtsola.com">个人站点</a> &nbsp;|&nbsp;
  📺 <a href="https://space.bilibili.com/736015">B站</a> &nbsp;|&nbsp;
  💬 微信：dtsola（技术交流 | 商务合作）
</p>

<p align="center">
  <img src="docs/产品文档/产品截图/个人二维码.png" alt="微信二维码" width="110">
  &nbsp;&nbsp;&nbsp;&nbsp;
  <img src="docs/运营文档/开发者交流群图.png" alt="开发者交流群" width="110">
  &nbsp;&nbsp;&nbsp;&nbsp;
  <img src="docs/运营文档/用户交流群图.png" alt="用户交流群" width="110">
</p>

<p align="center">
  <small>微信联系 &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp; 开发者交流群 &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp; 用户交流群</small>
</p>

- 项目主页：[GitHub](https://github.com/dtsola/xiaoyaoprivatebill-desktop)
- 问题反馈：[Issues](https://github.com/dtsola/xiaoyaoprivatebill-desktop/issues)

---

**Made with ❤️ by Xiaoyao Team**
