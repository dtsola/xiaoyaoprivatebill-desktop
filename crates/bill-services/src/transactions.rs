//! 交易记录列表：筛选、搜索、分页。
//!
//! 行为基准为 Web 版 `api/analysis.py::get_transactions`。

use bill_core::{Dataset, Transaction};
use chrono::{Datelike, Timelike};
use regex::RegexBuilder;
use serde_json::{json, Value};

/// 默认每页条数。
pub const DEFAULT_PER_PAGE: usize = 20;
/// 每页上限。
pub const MAX_PER_PAGE: usize = 100;

/// 交易记录查询条件。
#[derive(Debug, Default, Clone)]
pub struct TransactionQuery {
    /// 页码，从 1 开始。
    pub page: usize,
    /// 每页条数。
    pub per_page: usize,
    /// 年份。
    pub year: Option<i32>,
    /// 月份。
    pub month: Option<u32>,
    /// 日期 `YYYY-MM-DD`。
    pub date: Option<String>,
    /// 小时。
    pub hour: Option<u32>,
    /// 分类。
    pub category: Option<String>,
    /// 金额下限（含），0 视为未设置。
    pub min_amount: Option<f64>,
    /// 金额上限（含），0 视为未设置。
    pub max_amount: Option<f64>,
    /// 收/支。
    pub r#type: Option<String>,
    /// 正则搜索（忽略大小写），匹配商品说明/交易对方/交易分类。
    pub search: Option<String>,
}

/// 查询交易记录并返回 Web 版形状的响应。
pub fn list_transactions(dataset: &Dataset, query: &TransactionQuery) -> Value {
    let mut rows: Vec<&Transaction> = dataset.transactions.iter().collect();

    if let Some(kind) = query.r#type.as_deref() {
        rows.retain(|row| row.direction == kind);
    }

    if let Some(pattern) = query.search.as_deref() {
        let regex = match RegexBuilder::new(pattern).case_insensitive(true).build() {
            Ok(regex) => regex,
            Err(error) => {
                return json!({
                    "success": false,
                    "error": format!("获取交易记录失败: {error}")
                });
            }
        };
        rows.retain(|row| {
            regex.is_match(&row.description)
                || regex.is_match(&row.counterparty)
                || regex.is_match(&row.category)
        });
    }

    if let Some(year) = query.year {
        rows.retain(|row| row.time.year() == year);
    }
    if let Some(month) = query.month {
        rows.retain(|row| row.time.month() == month);
    }
    if let Some(date) = query.date.as_deref() {
        rows.retain(|row| row.date == date);
    }
    if let Some(hour) = query.hour {
        rows.retain(|row| row.time.hour() == hour);
    }
    if let Some(category) = query.category.as_deref() {
        rows.retain(|row| row.category == category);
    }
    if let Some(min) = query.min_amount.filter(|value| *value != 0.0) {
        rows.retain(|row| row.amount >= min);
    }
    if let Some(max) = query.max_amount.filter(|value| *value != 0.0) {
        rows.retain(|row| row.amount <= max);
    }

    rows.retain(|row| (row.direction == "收入" || row.direction == "支出") && !row.is_refund);
    rows.sort_by_key(|row| std::cmp::Reverse(row.time));

    let total_income: f64 = rows
        .iter()
        .filter(|row| row.direction == "收入")
        .map(|row| row.amount)
        .sum();
    let total_expense: f64 = rows
        .iter()
        .filter(|row| row.direction == "支出")
        .map(|row| row.amount)
        .sum();

    let per_page = if query.per_page == 0 {
        DEFAULT_PER_PAGE
    } else {
        query.per_page.min(MAX_PER_PAGE)
    };
    let total_records = rows.len();
    let total_pages = total_records.div_ceil(per_page);
    let page = query.page.max(1).min(total_pages.max(1));
    let start = (page - 1) * per_page;
    let end = (start + per_page).min(total_records);

    let items: Vec<Value> = rows[start..end]
        .iter()
        .map(|row| {
            json!({
                "time": row.time.format("%Y-%m-%d %H:%M:%S").to_string(),
                "description": row.description,
                "category": row.category,
                "type": row.direction,
                "amount": row.amount,
                "status": row.status,
                "counterparty": row.counterparty,
            })
        })
        .collect();

    json!({
        "success": true,
        "transactions": items,
        "summary": {
            "total_income": total_income,
            "total_expense": total_expense,
            "balance": total_income - total_expense,
        },
        "pagination": {
            "current_page": page,
            "per_page": per_page,
            "total_pages": total_pages,
            "total_records": total_records,
        }
    })
}
