//! MCP 运行状态：管理器 + 设置持久化。

use std::sync::{Arc, Mutex};

use bill_core::{DataPaths, DesktopSettings, McpSettings, Settings};
use bill_mcp::lifecycle::McpManager;
use serde_json::{json, Value};

/// 主管 MCP 生命周期的共享状态。
pub struct McpState {
    /// 数据运行时。
    pub runtime: Arc<bill_api::Runtime>,
    /// 生命周期管理器。
    pub manager: McpManager,
    settings: Mutex<Settings>,
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
