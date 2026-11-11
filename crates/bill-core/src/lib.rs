//! 小遥账单桌面版核心领域模型。
//!
//! P0 阶段只提供最小模型；P1 会补齐 Dataset 加载、文件条目与错误类型。

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// 统一交易记录。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub time: NaiveDateTime,
    pub month: String,
    pub date: String,
    pub direction: String,
    pub amount: f64,
    pub category: String,
    pub description: String,
    pub counterparty: String,
    pub payment_method: String,
    pub status: String,
    pub is_refund: bool,
    pub source: String,
}

/// 已导入的账单文件。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub size: u64,
    pub source: String,
    pub imported_at: String,
}

/// 解析后的数据集。
#[derive(Debug, Clone, Default)]
pub struct Dataset {
    pub transactions: Vec<Transaction>,
    pub files: Vec<FileEntry>,
}

/// 核心错误类型。
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("数据目录不可用: {0}")]
    DataDir(String),
    #[error("文件读写失败: {0}")]
    Io(String),
}
