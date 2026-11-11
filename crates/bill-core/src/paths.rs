//! 数据目录布局。
//!
//! `<app_data_dir>` 是唯一根目录，布局见技术方案 03 §8.1。

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::CoreError;

/// 应用数据目录及其派生路径。
#[derive(Debug, Clone)]
pub struct DataPaths {
    base: PathBuf,
}

impl DataPaths {
    /// 以应用数据根目录构造。
    pub fn new(base: PathBuf) -> Self {
        Self { base }
    }

    /// 根目录。
    pub fn base(&self) -> &Path {
        &self.base
    }

    /// `bills/`：账单唯一真源。
    pub fn bills_dir(&self) -> PathBuf {
        self.base.join("bills")
    }

    /// `trash/`：回收站，永不自动清理。
    pub fn trash_dir(&self) -> PathBuf {
        self.base.join("trash")
    }

    /// `snapshots/`：快照，永不自动清理。
    pub fn snapshots_dir(&self) -> PathBuf {
        self.base.join("snapshots")
    }

    /// `tmp/`：导入暂存与原子写。
    pub fn tmp_dir(&self) -> PathBuf {
        self.base.join("tmp")
    }

    /// `logs/`：日志目录。
    pub fn logs_dir(&self) -> PathBuf {
        self.base.join("logs")
    }

    /// `settings.json`。
    pub fn settings_file(&self) -> PathBuf {
        self.base.join("settings.json")
    }

    /// `session.json`。
    pub fn session_file(&self) -> PathBuf {
        self.base.join("session.json")
    }

    /// `meta.json`。
    pub fn meta_file(&self) -> PathBuf {
        self.base.join("meta.json")
    }

    /// `mcp_token`。
    pub fn mcp_token_file(&self) -> PathBuf {
        self.base.join("mcp_token")
    }

    /// `audit.jsonl`。
    pub fn audit_file(&self) -> PathBuf {
        self.base.join("audit.jsonl")
    }

    /// 创建目录布局，幂等。
    pub fn ensure_layout(&self) -> Result<(), CoreError> {
        for dir in [
            self.base.clone(),
            self.bills_dir(),
            self.trash_dir(),
            self.snapshots_dir(),
            self.tmp_dir(),
            self.logs_dir(),
        ] {
            fs::create_dir_all(&dir).map_err(|error| {
                CoreError::DataDir(format!("创建目录 {} 失败: {error}", dir.display()))
            })?;
        }
        Ok(())
    }
}
