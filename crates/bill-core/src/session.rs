//! 会话状态（`session.json`）。
//!
//! 桌面端没有 Cookie，用本地文件保留 Web 版的会话语义：
//! 首次导入写入 `user_local`，进入演示写入 `demo_user`，退出演示清除两者。

use serde::{Deserialize, Serialize};

use crate::{write_atomic, CoreError, DataPaths};

/// 当前 `session.json` 结构版本。
const SCHEMA_VERSION: u32 = 1;

/// 本地会话状态。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Session {
    /// 结构版本。
    #[serde(default = "schema_version")]
    pub schema_version: u32,
    /// `user_local` / `demo_user`；无会话时为 `None`。
    #[serde(default)]
    pub user_id: Option<String>,
    /// 是否处于演示模式。
    #[serde(default)]
    pub is_demo: bool,
    /// 会话开始时间（RFC 3339）；演示模式为空。
    #[serde(default)]
    pub session_start: Option<String>,
}

fn schema_version() -> u32 {
    SCHEMA_VERSION
}

impl Default for Session {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            user_id: None,
            is_demo: false,
            session_start: None,
        }
    }
}

impl Session {
    /// 读取会话；文件缺失或损坏时返回默认值。
    pub fn load(paths: &DataPaths) -> Self {
        std::fs::read(paths.session_file())
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    /// 原子写回。
    pub fn save(&self, paths: &DataPaths) -> Result<(), CoreError> {
        let bytes = serde_json::to_vec_pretty(self)
            .map_err(|error| CoreError::Io(format!("序列化 session.json 失败: {error}")))?;
        write_atomic(&paths.session_file(), &bytes)
    }

    /// 是否已有会话（`user_id` 存在）。
    pub fn is_active(&self) -> bool {
        self.user_id.is_some()
    }

    /// 进入演示模式：写入 `demo_user` 与 `is_demo=true`。
    pub fn enter_demo(paths: &DataPaths) -> Result<Self, CoreError> {
        let mut session = Self::load(paths);
        session.is_demo = true;
        session.user_id = Some("demo_user".to_owned());
        session.save(paths)?;
        Ok(session)
    }

    /// 退出演示模式：清除 `is_demo` 与 `user_id`。
    pub fn exit_demo(paths: &DataPaths) -> Result<Self, CoreError> {
        let mut session = Self::load(paths);
        session.is_demo = false;
        session.user_id = None;
        session.save(paths)?;
        Ok(session)
    }

    /// 导入成功后激活本地会话；已存在会话（含演示）时保持不变。
    pub fn mark_import(paths: &DataPaths) -> Result<Self, CoreError> {
        let mut session = Self::load(paths);
        if session.user_id.is_none() {
            session.user_id = Some("user_local".to_owned());
            session.session_start = Some(chrono::Local::now().to_rfc3339());
            session.save(paths)?;
        }
        Ok(session)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_session_is_inactive() {
        let session = Session::default();
        assert!(!session.is_active());
        assert!(!session.is_demo);
        assert_eq!(session.schema_version, SCHEMA_VERSION);
    }
}
