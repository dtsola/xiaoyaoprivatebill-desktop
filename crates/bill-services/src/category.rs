//! 分类页基础数据：分类支出与可用年月。
//!
//! 行为基准为 Web 版 `api/analysis.py` 的 `category_expenses`
//! 与 `category_available_dates`。

use std::collections::{BTreeMap, BTreeSet};

use bill_core::Dataset;
use chrono::Datelike;
use serde_json::{json, Value};

/// 分类支出统计：仅支出行（含退款，复刻 D7），按金额降序。
pub fn category_expenses(dataset: &Dataset) -> Value {
    let mut grouped: BTreeMap<String, f64> = BTreeMap::new();
    for row in &dataset.transactions {
        if row.direction == "支出" {
            *grouped.entry(row.category.clone()).or_insert(0.0) += row.amount;
        }
    }

    let mut items: Vec<(String, f64)> = grouped.into_iter().collect();
    items.sort_by(|left, right| right.1.total_cmp(&left.1).then(left.0.cmp(&right.0)));

    let categories: Vec<String> = items.iter().map(|(name, _)| name.clone()).collect();
    let amounts: Vec<f64> = items.iter().map(|(_, amount)| *amount).collect();
    json!({ "categories": categories, "amounts": amounts })
}

/// 分类页可用年月：年份降序，月份升序。
pub fn category_available_dates(dataset: &Dataset) -> Value {
    let mut grouped: BTreeMap<i32, BTreeSet<u32>> = BTreeMap::new();
    for row in &dataset.transactions {
        grouped
            .entry(row.time.year())
            .or_default()
            .insert(row.time.month());
    }

    let years: Vec<i32> = grouped.keys().rev().copied().collect();
    let months: BTreeMap<String, Vec<u32>> = grouped
        .into_iter()
        .map(|(year, values)| (year.to_string(), values.into_iter().collect()))
        .collect();

    json!({ "years": years, "months": months })
}
