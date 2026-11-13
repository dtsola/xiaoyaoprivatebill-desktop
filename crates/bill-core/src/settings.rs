//! `settings.json` 的模型与读写。
//!
//! 结构见技术方案 03 §8.2；窗口状态由 Tauri 插件单独持久化，不写这里。

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::CoreError;
use crate::fsutil::write_atomic;

/// 当前设置 schema 版本。
pub const SCHEMA_VERSION: u32 = 1;

/// 应用设置根结构。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    /// 设置结构版本，用于后续迁移。
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    /// 桌面行为。
    #[serde(default)]
    pub desktop: DesktopSettings,
    /// MCP 服务。
    #[serde(default)]
    pub mcp: McpSettings,
    /// 数据策略。
    #[serde(default)]
    pub data: DataSettings,
}

fn default_schema_version() -> u32 {
    SCHEMA_VERSION
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            desktop: DesktopSettings::default(),
            mcp: McpSettings::default(),
            data: DataSettings::default(),
        }
    }
}

impl Settings {
    /// 读取设置；文件不存在时返回默认值，解析失败时报错。
    pub fn load(path: &Path) -> Result<Self, CoreError> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let bytes = std::fs::read(path).map_err(|error| {
            CoreError::Settings(format!("读取 {} 失败: {error}", path.display()))
        })?;
        serde_json::from_slice(&bytes).map_err(|error| {
            CoreError::Settings(format!("{} 不是合法 JSON: {error}", path.display()))
        })
    }

    /// 原子写入设置。
    pub fn save(&self, path: &Path) -> Result<(), CoreError> {
        let bytes = serde_json::to_vec_pretty(self)
            .map_err(|error| CoreError::Settings(format!("序列化设置失败: {error}")))?;
        write_atomic(path, &bytes)
    }
}

/// 桌面行为设置。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DesktopSettings {
    /// 关闭窗口时最小化到托盘。
    #[serde(default = "default_true")]
    pub close_to_tray: bool,
    /// 是否已展示过「关闭到托盘」提示。
    #[serde(default)]
    pub close_hint_shown: bool,
    /// 界面语言偏好：`system` / `zh-CN` / `en-US`。
    #[serde(default = "default_language")]
    pub language: String,
}

fn default_language() -> String {
    "system".to_owned()
}

fn default_true() -> bool {
    true
}

impl Default for DesktopSettings {
    fn default() -> Self {
        Self {
            close_to_tray: true,
            close_hint_shown: false,
            language: default_language(),
        }
    }
}

/// MCP 服务设置。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpSettings {
    /// 是否启用，默认关闭。
    #[serde(default)]
    pub enabled: bool,
    /// 监听端口，仅回环。
    #[serde(default = "default_mcp_port")]
    pub port: u16,
    /// 最近一次启动失败原因。
    #[serde(default)]
    pub last_error: Option<String>,
}

fn default_mcp_port() -> u16 {
    17650
}

impl Default for McpSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            port: 17650,
            last_error: None,
        }
    }
}

/// 数据策略设置。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataSettings {
    /// 文件名清洗是否与 werkzeug 完全一致（复刻 D3）。
    #[serde(default = "default_true")]
    pub parity_filenames: bool,
    /// 清空前是否自动创建快照。
    #[serde(default = "default_true")]
    pub snapshot_before_clear: bool,
}

impl Default for DataSettings {
    fn default() -> Self {
        Self {
            parity_filenames: true,
            snapshot_before_clear: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_language_follows_system() {
        let settings = Settings::default();
        assert_eq!(settings.desktop.language, "system");
    }

    #[test]
    fn legacy_settings_without_language_use_system() {
        let legacy = r#"{
            "schema_version": 1,
            "desktop": {"close_to_tray": false, "close_hint_shown": true},
            "mcp": {"enabled": false, "port": 17650},
            "data": {"parity_filenames": true, "snapshot_before_clear": true}
        }"#;
        let settings: Settings = serde_json::from_str(legacy).expect("旧设置应可解析");
        assert_eq!(settings.desktop.language, "system");
    }

    #[test]
    fn language_round_trips_through_json() {
        let mut settings = Settings::default();
        settings.desktop.language = "en-US".to_owned();
        let text = serde_json::to_string(&settings).unwrap();
        let parsed: Settings = serde_json::from_str(&text).unwrap();
        assert_eq!(parsed.desktop.language, "en-US");
    }
}
