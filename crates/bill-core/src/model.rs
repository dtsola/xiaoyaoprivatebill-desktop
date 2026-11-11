//! 领域模型：交易记录、文件条目、数据集。
//!
//! 字段与 Web 版 DataFrame 列一一对应，见技术方案 03 §6.1。

use chrono::{Datelike, NaiveDateTime};
use serde::{Deserialize, Serialize};

/// 统一交易记录。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    /// 交易时间。
    pub time: NaiveDateTime,
    /// 月份 `YYYY-MM`。
    pub month: String,
    /// 日期 `YYYY-MM-DD`。
    pub date: String,
    /// 收/支。
    pub direction: String,
    /// 金额，支出为负。
    pub amount: f64,
    /// 交易分类。
    pub category: String,
    /// 商品说明。
    pub description: String,
    /// 交易对方 / 商户。
    pub counterparty: String,
    /// 收/付款方式。
    pub payment_method: String,
    /// 交易状态。
    pub status: String,
    /// 是否退款。
    pub is_refund: bool,
    /// 来源：支付宝 / 微信。
    pub source: String,
}

/// 已导入的账单文件。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileEntry {
    /// 文件名。
    pub name: String,
    /// 字节数。
    pub size: u64,
    /// 来源：支付宝 / 微信。
    pub source: String,
    /// 导入时间 `YYYY-MM-DD HH:MM:SS`。
    pub imported_at: String,
}

/// 解析后的数据集。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Dataset {
    /// 全部交易，按导入顺序稳定排序。
    pub transactions: Vec<Transaction>,
    /// 已导入文件。
    pub files: Vec<FileEntry>,
}

impl Dataset {
    /// 是否没有任何交易。
    pub fn is_empty(&self) -> bool {
        self.transactions.is_empty()
    }

    /// 已导入文件数。
    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    /// 可用年份，去重并按降序。
    pub fn available_years(&self) -> Vec<i32> {
        let mut years: Vec<i32> = self
            .transactions
            .iter()
            .map(|item| item.time.year())
            .collect();
        years.sort_unstable_by(|a, b| b.cmp(a));
        years.dedup();
        years
    }
}
