//! MCP 运行状态：管理器 + 设置持久化。

use std::sync::{Arc, Mutex};

use bill_core::{DataPaths, DesktopSettings, McpSettings, Settings};
use bill_mcp::lifecycle::McpManager;
use serde_json::{json, Value};
use tauri::menu::MenuItem;
use tauri::Wry;

use crate::shell_i18n::{normalize_locale, shell_labels};

/// 主管 MCP 生命周期的共享状态。
pub struct McpState {
    /// 数据运行时。
    pub runtime: Arc<bill_api::Runtime>,
    /// 生命周期管理器。
    pub manager: McpManager,
    settings: Mutex<Settings>,
    tray_item: Mutex<Option<MenuItem<Wry>>>,
    /// 当前托盘 / 窗口语言（已归一为 zh-CN / en-US）。
    shell_locale: Mutex<String>,
}

impl McpState {
    /// 从 `settings.json` 构造。
    pub fn new(runtime: Arc<bill_api::Runtime>, paths: &DataPaths) -> Self {
        let settings = Settings::load(&paths.settings_file()).unwrap_or_default();
        let manager = McpManager::new(runtime.clone(), settings.mcp.port);
        Self {
            runtime,
            manager,
            settings: Mutex::new(settings),
            tray_item: Mutex::new(None),
            shell_locale: Mutex::new("zh-CN".to_owned()),
        }
    }

    /// 当前托盘 / 窗口语言。
    pub fn shell_locale(&self) -> String {
        self.shell_locale
            .lock()
            .map(|guard| guard.clone())
            .unwrap_or_else(|_| "zh-CN".to_owned())
    }

    /// 更新托盘 / 窗口语言（不落盘，落盘由 `set_language` 命令负责）。
    pub fn set_shell_locale(&self, locale: &str) {
        if let Ok(mut guard) = self.shell_locale.lock() {
            *guard = normalize_locale(locale).to_owned();
        }
    }

    /// 记录托盘里的 MCP 菜单项，供设置页切换后同步文案。
    pub fn set_tray_item(&self, item: MenuItem<Wry>) {
        if let Ok(mut guard) = self.tray_item.lock() {
            *guard = Some(item);
        }
    }

    /// 按当前启用状态与语言刷新托盘菜单文案。
    pub fn sync_tray_label(&self) {
        let labels = shell_labels(&self.shell_locale());
        let label = if self.enabled() {
            labels.mcp_on
        } else {
            labels.mcp_off
        };
        if let Ok(guard) = self.tray_item.lock() {
            if let Some(item) = guard.as_ref() {
                let _ = item.set_text(label);
            }
        }
    }

    /// 当前 MCP 设置快照。
    pub fn mcp_settings(&self) -> McpSettings {
        self.settings
            .lock()
            .map(|guard| guard.mcp.clone())
            .unwrap_or_default()
    }

    /// 是否已启用（以设置为准，不代表正在运行）。
    pub fn enabled(&self) -> bool {
        self.mcp_settings().enabled
    }

    /// 当前桌面设置快照。
    pub fn desktop_settings(&self) -> DesktopSettings {
        self.settings
            .lock()
            .map(|guard| guard.desktop.clone())
            .unwrap_or_default()
    }

    /// 更新桌面设置并落盘。
    pub fn update_desktop(&self, change: impl FnOnce(&mut DesktopSettings)) -> Result<(), String> {
        let mut guard = self
            .settings
            .lock()
            .map_err(|_| "设置状态不可用".to_owned())?;
        change(&mut guard.desktop);
        guard
            .save(&self.runtime.paths.settings_file())
            .map_err(|error| error.to_string())
    }

    /// 数据目录路径（展示用）。
    pub fn data_dir(&self) -> String {
        self.runtime.paths.base().display().to_string()
    }

    /// 更新 MCP 设置并落盘。
    pub fn update_mcp(&self, change: impl FnOnce(&mut McpSettings)) -> Result<(), String> {
        let mut guard = self
            .settings
            .lock()
            .map_err(|_| "设置状态不可用".to_owned())?;
        change(&mut guard.mcp);
        guard
            .save(&self.runtime.paths.settings_file())
            .map_err(|error| error.to_string())
    }

    /// 状态响应（供命令与前端使用）。
    pub fn status(&self) -> Value {
        let mcp = self.mcp_settings();
        json!({
            "enabled": mcp.enabled,
            "running": self.manager.is_running(),
            "port": mcp.port,
            "endpoint": format!("http://127.0.0.1:{}/mcp", mcp.port),
            "last_error": mcp.last_error,
        })
    }
}
