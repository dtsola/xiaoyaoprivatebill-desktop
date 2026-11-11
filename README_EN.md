# Xiaoyao Private Bill Analyzer – Desktop

> A privacy-first desktop app for personal bill analysis: fully offline, locally persistent, and callable by AI agents through MCP.

[![Tauri](https://img.shields.io/badge/Tauri-2.x-24C8DB.svg)](https://tauri.app/)
[![Rust](https://img.shields.io/badge/Rust-1.95+-orange.svg)](https://www.rust-lang.org/)
[![Vue](https://img.shields.io/badge/Vue-3.5+-brightgreen.svg)](https://vuejs.org/)
[![Windows](https://img.shields.io/badge/Windows-10%2F11%20x64-blue.svg)](https://www.microsoft.com/windows)
[![macOS](https://img.shields.io/badge/macOS-11%2B-lightgrey.svg)](https://www.apple.com/macos/)
[![MCP](https://img.shields.io/badge/MCP-14%20tools-purple.svg)](https://modelcontextprotocol.io/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

English | [简体中文](README.md)

<p align="center">
  <img src="docs/产品文档/产品截图/宣传海报图.png" alt="Xiaoyao Private Bill Analyzer Desktop Poster">
</p>

> **Current status**: design complete, implementation not started (v0.1.0 in development). Installers are not published yet. For now, please use [Development & Build](#8-development--build).

---

## 1. Introduction

<p align="center">
  <img src="docs/产品文档/logos/logo.png" alt="Xiaoyao Private Bill Analyzer Desktop" width="200">
</p>

Xiaoyao Private Bill Analyzer – Desktop is a **privacy-first** personal bill analysis tool. It parses Alipay and WeChat bills, provides multi-dimensional visual analysis, and ships with a built-in MCP server so AI agents such as Claude Code and Cursor can query your bills directly.

**Key features**:

- 🔒 **Fully offline** – no outbound requests, no telemetry, no CDN, no auto-update
- 💾 **Local persistence** – bills stay in the local app data directory; clear them manually at any time
- 📊 **Multi-dimensional analysis** – yearly, monthly, category, time, and consumption insights
- 📁 **Multiple formats** – Alipay CSV, WeChat CSV/XLSX
- 🤖 **Agent-callable** – 14 built-in MCP tools for natural-language bill queries
- 🖥️ **Desktop-native** – system tray, single instance, window-state memory, native file import
- 🗑️ **Recoverable deletes** – trash, snapshots, and audit log reduce accidental-loss risk
- 🧬 **Same origin as the Web edition** – identical visuals and statistics across 9 pages, with a Rust backend rewrite

---

## 2. Feature Preview

> The screens below match the Web edition. Screenshots of the desktop settings group, tray, and MCP configuration will be added before release.

### Upload Bills - One-Click Import

<table>
<tr>
<td width="50%"><img src="docs/产品文档/产品截图/08-上传账单.png" alt="Upload Bills"></td>
<td width="50%">
<ul>
<li>📤 Native file dialog with multi-select and full-window drag & drop</li>
<li>📋 Auto-detect Alipay/WeChat bill formats</li>
<li>⚡ Local parsing, nothing uploaded to any server</li>
</ul>
</td>
</tr>
</table>

### Homepage - Feature Dashboard

<table>
<tr>
<td width="50%"><img src="docs/产品文档/产品截图/01-首页.png" alt="Homepage"></td>
<td width="50%">
<ul>
<li>🎯 All-in-one analysis for Alipay and WeChat bills</li>
<li>📊 Four capability cards: analysis, privacy, trends, smart search</li>
<li>🚀 Import bills or enter demo mode</li>
</ul>
</td>
</tr>
</table>

### Yearly Overview - At a Glance

<table>
<tr>
<td width="50%"><img src="docs/产品文档/产品截图/02-年度总览.png" alt="Yearly Overview"></td>
<td width="50%">
<ul>
<li>📊 Yearly income and expense overview</li>
<li>📈 Monthly trends and category breakdown</li>
<li>📖 Yearly story review</li>
</ul>
</td>
</tr>
</table>

### Monthly Analysis - Detailed Comparison

<table>
<tr>
<td width="50%"><img src="docs/产品文档/产品截图/03-月度分析.png" alt="Monthly Analysis"></td>
<td width="50%">
<ul>
<li>📅 Monthly income, expense, and month-over-month change</li>
<li>🗓️ Daily spending calendar</li>
<li>📉 Monthly comparison and category details</li>
</ul>
</td>
</tr>
</table>

### Category Analysis - Spending Breakdown

<table>
<tr>
<td width="50%"><img src="docs/产品文档/产品截图/04-分类分析.png" alt="Category Analysis"></td>
<td width="50%">
<ul>
<li>🏷️ Category share and ranking</li>
<li>📊 Category trends and hourly distribution</li>
<li>📝 Category transaction details</li>
</ul>
</td>
</tr>
</table>

### Time Analysis - Spending Patterns

<table>
<tr>
<td width="50%"><img src="docs/产品文档/产品截图/05-时间分析.png" alt="Time Analysis"></td>
<td width="50%">
<ul>
<li>⏰ 24-hour spending distribution</li>
<li>🌡️ Calendar heatmap</li>
<li>📈 Weekday vs. weekend comparison</li>
</ul>
</td>
</tr>
</table>

### Consumption Insights - Smart Analytics

<table>
<tr>
<td width="50%"><img src="docs/产品文档/产品截图/06-消费洞察-01.png" alt="Consumption Insights 1"></td>
<td width="50%"><img src="docs/产品文档/产品截图/06-消费洞察-02.png" alt="Consumption Insights 2"></td>
</tr>
<tr>
<td width="50%"><img src="docs/产品文档/产品截图/06-消费洞察-03.png" alt="Consumption Insights 3"></td>
<td width="50%">
<ul>
<li>💡 Spending habits and profiles</li>
<li>🔍 Anomaly detection and latte factor</li>
<li>📊 Late-night spending, subscriptions, Engel coefficient</li>
</ul>
</td>
</tr>
</table>

### Transaction Records - Detailed History

<table>
<tr>
<td width="50%"><img src="docs/产品文档/产品截图/07-交易记录.png" alt="Transaction Records"></td>
<td width="50%">
<ul>
<li>📋 Complete transaction history</li>
<li>🔍 Search and multi-dimensional filters</li>
<li>📄 Paginated browsing</li>
</ul>
</td>
</tr>
</table>

<!-- TODO(P6): add 09-桌面.png (desktop settings group) -->
<!-- TODO(P6): add 10-系统托盘.png (system tray menu) -->
<!-- TODO(P6): add 11-MCP配置.png (MCP toggle and config copy) -->

---

## 3. Installation

### 3.1 Windows 10 / 11 (x64)

1. Download `*-setup.exe` from [GitHub Releases](https://github.com/dtsola/xiaoyaoprivatebill-desktop/releases).
2. Double-click and follow the installer; it installs per-user and does not require administrator rights.
3. Launch "小遥账单桌面版" from the Start menu.

> Windows 10 requires the WebView2 Runtime (Windows 11 ships with it). If the app fails to start, install [WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/) first.

### 3.2 macOS 11+ (Intel / Apple Silicon)

1. Download the matching `.dmg` from [GitHub Releases](https://github.com/dtsola/xiaoyaoprivatebill-desktop/releases):
   - Intel: `*_x64.dmg`
   - Apple Silicon: `*_aarch64.dmg`
2. Open the dmg and drag the app into Applications.
3. **First launch**: right-click the app → "Open"; or go to "System Settings → Privacy & Security" and click "Open Anyway".
4. If macOS 15 still blocks it, run:

```bash
xattr -dr com.apple.quarantine "/Applications/小遥账单桌面版.app"
```

> This version is **not signed or notarized**, hence the warning. Signing and notarization will be added in a later version.

### 3.3 System Requirements

| Item | Requirement |
|---|---|
| Windows | Windows 10 / 11 x64; WebView2 installed |
| macOS | macOS 11+; Intel or Apple Silicon |
| Disk | Installer about 20MB; data directory grows with bills |
| Network | **Not required**; fully usable offline |

### 3.4 Data Directory and Uninstall

| Platform | Data directory |
|---|---|
| Windows | `%APPDATA%\xiaoyaobill-desktop` |
| macOS | `~/Library/Application Support/xiaoyaobill-desktop` |

- Uninstalling the app does **not** delete the data directory; delete it manually if needed.
- To back up, copy the entire data directory; `bills/` is the single source of truth.

---

## 4. Quick Start

### 4.1 Get Bill Files

**Alipay bills**:

1. Open the Alipay app → Bills.
2. Tap "Common Questions" in the top-right → Export Bills.
3. Choose "Personal Bills" → date range → CSV format.
4. Enter your email and wait for the bill file.

**WeChat bills**:

1. Open WeChat → Me → Services → Wallet → "Bills" in the top-right.
2. Tap "Common Questions" → Download Bills.
3. Choose a date range → enter your email → choose CSV or XLSX.

### 4.2 Import and Analyze

1. Open the app and click "Upload Bills" or go to "Settings".
2. Click the Alipay/WeChat upload area to select files, or drag bills into the window.
3. Wait for parsing; then explore yearly, monthly, category, time, insights, and transaction pages.
4. To try it first, click "Try Demo Data" to load the built-in 7,396 sample records.

### 4.3 Demo Mode

- All 9 pages are browsable in demo mode.
- Demo mode **blocks** imports, deletes, clears, and all MCP write operations.
- Exit demo mode to return to real data.

---

## 5. Using MCP

### 5.1 Start the Service

1. Open "Settings → Desktop → MCP Service".
2. Turn on the switch; the status shows "Running".
3. The endpoint is fixed at `http://127.0.0.1:17650/mcp` and only listens on loopback.
4. Click "Copy MCP Config" and paste it into your MCP client.

> MCP is off by default. Once enabled, local programs can read all your bills; whether data leaves this machine depends on the calling agent.

### 5.2 Client Configuration

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

Claude Code can also add it directly:

```bash
claude mcp add --transport http xiaoyao-bill http://127.0.0.1:17650/mcp \
  --header "Authorization: Bearer <TOKEN>"
```

### 5.3 The 14 Tools

| Type | Tools |
|---|---|
| Read (10) | `list_bills`, `get_overview`, `get_summary`, `get_yearly_analysis`, `get_monthly_analysis`, `get_category_analysis`, `search_transactions`, `get_time_analysis`, `get_insights`, `get_audit_log` |
| Write (4) | `import_bill`, `delete_bill`, `restore_bill`, `clear_data` |

Write operations are guarded: `import_bill` accepts only absolute paths and ≤16MB; `delete_bill` soft-deletes to trash; `clear_data` requires a snapshot first; every write is audited.

### 5.4 Security Notes

- The token is a 32-byte random value; regenerate it in Settings to invalidate old connections.
- The server listens only on `127.0.0.1`; it never listens on the LAN or public internet.
- Trash, snapshots, and audit logs are **never cleaned automatically**; clear them manually in Settings.
- Keep MCP off if you do not need agent access.

---

## 6. Privacy

| Item | Description |
|---|---|
| Network | The app makes no outbound requests; no telemetry, crash reporting, or auto-update |
| Fonts and icons | Font Awesome is bundled locally; no CDN |
| Data storage | Stored only in the local app data directory; never uploaded |
| Data cleanup | Deletes go to trash; clears create a snapshot; writes are audited; all cleanup is manual |
| External links | Opened in the system browser |
| MCP | Off by default; loopback only; Bearer token; writes audited; whether data leaves the machine depends on the caller |
| Uninstall | The data directory is not deleted; remove it manually if needed |

---

## 7. Migrating from the Web Edition

- The desktop edition is a **separate product**; it does not read or migrate the Web edition's data directory.
- Both editions share the same visuals and statistics, including known defects (such as Alipay refund detection).
- To migrate, simply re-import the original bill files in the desktop app.
- The Web edition is frozen and receives security fixes only; new features go to the desktop edition.
- Same-origin Web edition: [dtsola/xiaoyaoprivatebill](https://github.com/dtsola/xiaoyaoprivatebill).

---

## 8. Development & Build

### 8.1 Prerequisites

| Tool | Version |
|---|---|
| Rust | 1.95+ (MSVC toolchain on Windows) |
| Node.js | 20 LTS or 24 |
| npm | 10+ |
| Visual Studio Build Tools | 2022 (Windows, with VC Tools) |
| WebView2 | Windows 10/11 |
| Python | 3.10 (golden reconciliation only; not shipped) |

### 8.2 Local Development

```bash
# 1. Clone the repository
git clone https://github.com/dtsola/xiaoyaoprivatebill-desktop.git
cd xiaoyaoprivatebill-desktop

# 2. Install dependencies
npm install
npm --prefix frontend install

# 3. Start development mode (Vite + Tauri)
npm run tauri dev

# 4. Frontend-only development
npm --prefix frontend run dev
```

### 8.3 Build Installers

```bash
# Build the frontend
npm --prefix frontend run build

# Build the desktop installer (Windows NSIS)
npm run tauri build
# Output: src-tauri/target/release/bundle/nsis/*.exe
```

macOS builds are produced per architecture via GitHub Actions as `x86_64.dmg` and `aarch64.dmg`; the workflow file is `.github/workflows/macos-build.yml`.

### 8.4 Quality and Reconciliation

```bash
cargo fmt --check
cargo clippy --workspace -- -D warnings
cargo test --workspace

# golden reconciliation (requires Python 3.10 with locked dependencies)
python tools/golden/generate.py
git diff --exit-code fixtures/expected

# Frontend frozen-baseline check (only 7 files may differ)
node scripts/check-frontend-baseline.mjs
```

---

## 9. Documentation

| Document | Description |
|---|---|
| [Market Requirements (MRD)](docs/00-mrd.md) | Problem, users, competitors, success metrics |
| [Product Requirements (PRD)](docs/01-prd.md) | Features, desktop behavior, MCP, pages, acceptance |
| [Prototype & Visual Standard](docs/02-原型+视觉.md) | Page baseline, components, desktop group, import UX, copy, visual acceptance |
| [Technical Specification](docs/03-技术方案文档.md) | Tauri + Rust architecture, 27 APIs, storage, security, packaging |
| [Development Task List](docs/04-开发任务清单.md) | 61 tasks, effort, milestones, quality gates |
| [Development Schedule](docs/05-开发排期.md) | D1–D22, critical path, risks |
| [Development Progress](docs/开发进度.md) | Current progress, Web baseline, technical debt |
| [Desktop Research](docs/桌面端-调研方案/) | Feasibility, PRD, prototype, technical plan, implementation steps (read-only archive) |

---

## 10. FAQ

### Q: Windows 10 fails to start with a missing WebView2 error?

A: Install the [WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/) and retry. Windows 11 usually ships with it.

### Q: macOS says the developer cannot be verified?

A: Right-click the app → "Open"; or go to "System Settings → Privacy & Security" and click "Open Anyway". You can also run the `xattr` command above.

### Q: MCP fails to start because the port is in use?

A: Port `17650` is occupied by another program. Close it and retry, or keep MCP off; GUI features are unaffected.

### Q: What should I check if an import fails?

A: Confirm:
- Alipay bills are `.csv`, encoded as GBK or UTF-8;
- WeChat bills are `.csv` or `.xlsx`;
- Each file is no larger than 16MB;
- **File names use ASCII characters** (Chinese file names may be sanitized and become unrecognizable; this behavior is preserved for parity with the Web edition).

### Q: What happens if I import the same file twice?

A: Files are not deduplicated by content; same-name files get `_1`, `_2` suffixes, and duplicate transactions are counted twice. This is a known behavior shared with the Web edition.

### Q: Is data uploaded to any server?

A: No. The app makes no outbound requests. Data can only leave the machine if you enable MCP and an agent calls it.

### Q: How do I back up or migrate data?

A: Copy the entire data directory (see [3.4](#34-data-directory-and-uninstall)). `bills/` is the source of truth; `trash/`, `snapshots/`, and `audit.jsonl` hold trash, snapshots, and audit logs.

### Q: Is Linux or mobile supported?

A: No. This release supports Windows 10/11 x64 and macOS 11+ only.

---

## 11. Acknowledgments

- The frontend and product design share their origin with [dtsola/xiaoyaoprivatebill](https://github.com/dtsola/xiaoyaoprivatebill) (Web edition, frozen).
- The Web edition was originally refactored from [alipay_record_analysis](https://github.com/Hessel2333/alipay_record_analysis); thanks to **Hessel2333**.
- The desktop edition is built with [Tauri](https://tauri.app/), [Rust](https://www.rust-lang.org/), [Vue 3](https://vuejs.org/), [ECharts](https://echarts.apache.org/), and [rmcp](https://github.com/modelcontextprotocol/rust-sdk).

---

## 12. Contributing

Contributions, issue reports, and suggestions are welcome.

### Workflow

1. Fork this repository.
2. Create a feature branch: `git checkout -b feature/AmazingFeature`.
3. Commit your changes: `git commit -m 'feat: add some feature'`.
4. Push the branch: `git push origin feature/AmazingFeature`.
5. Open a Pull Request.

### Code Standards

- Rust follows `cargo fmt` and `cargo clippy -- -D warnings`.
- The frontend may modify only the 7 baseline files; everything else stays frozen.
- Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/).
- See [.claude/CLAUDE.md](.claude/CLAUDE.md) for details.

### Pre-Submit Checks

```bash
cargo fmt --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
node scripts/check-frontend-baseline.mjs
```

---

## 13. License & Contact

This project is licensed under the [MIT License](LICENSE).

<p align="center">
  <img src="docs/产品文档/产品截图/作者头像.jpg" alt="dtsola" width="100" height="100" style="border-radius: 50%;">
</p>

<p align="center">
  <b>dtsola</b> — IT Solution Architect | Indie Solo Practitioner
</p>

<p align="center">
  🌐 <a href="https://www.dtsola.com">Personal Site</a> &nbsp;|&nbsp;
  📺 <a href="https://space.bilibili.com/736015">Bilibili</a> &nbsp;|&nbsp;
  💬 WeChat: dtsola (Tech Exchange | Business Cooperation)
</p>

<p align="center">
  <img src="docs/产品文档/产品截图/个人二维码.png" alt="WeChat QR Code" width="110">
  &nbsp;&nbsp;&nbsp;&nbsp;
  <img src="docs/运营文档/开发者交流群图.png" alt="Developer Community" width="110">
  &nbsp;&nbsp;&nbsp;&nbsp;
  <img src="docs/运营文档/用户交流群图.png" alt="User Community" width="110">
</p>

<p align="center">
  <small>WeChat Contact &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp; Developer Community &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp; User Community</small>
</p>

- Project homepage: [GitHub](https://github.com/dtsola/xiaoyaoprivatebill-desktop)
- Issue tracker: [Issues](https://github.com/dtsola/xiaoyaoprivatebill-desktop/issues)

---

**Made with ❤️ by Xiaoyao Team**
