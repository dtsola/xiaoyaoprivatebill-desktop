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

/// 分类分析（列表分支）。
///
/// `category` 为空时返回分类排行；按 0.7×金额排名 + 0.3×笔数排名升序，
/// 排名采用 pandas 的 average 规则（并列取平均名次）。
pub fn category_analysis(dataset: &Dataset, category: Option<&str>, year: Option<i32>) -> Value {
    if let Some(category) = category {
        return json!({ "error": format!("未找到分类 \"{category}\" 的数据") });
    }

    let rows: Vec<&bill_core::Transaction> = dataset
        .transactions
        .iter()
        .filter(|row| row.direction == "支出" && !row.is_refund)
        .filter(|row| year.is_none_or(|value| row.time.year() == value))
        .collect();

    let mut grouped: BTreeMap<String, (f64, usize)> = BTreeMap::new();
    for row in &rows {
        let entry = grouped.entry(row.category.clone()).or_insert((0.0, 0));
        entry.0 += row.amount;
        entry.1 += 1;
    }

    let names: Vec<String> = grouped.keys().cloned().collect();
    let totals: Vec<f64> = grouped.values().map(|(total, _)| round2(*total)).collect();
    let counts: Vec<usize> = grouped.values().map(|(_, count)| *count).collect();
    let averages: Vec<f64> = grouped
        .values()
        .map(|(total, count)| round2(total / *count as f64))
        .collect();

    let amount_ranks = average_ranks(&totals, true);
    let count_ranks = average_ranks(&counts_as_f64(&counts), true);
    let scores: Vec<f64> = amount_ranks
        .iter()
        .zip(&count_ranks)
        .map(|(amount, count)| 0.7 * amount + 0.3 * count)
        .collect();

    let mut order: Vec<usize> = (0..names.len()).collect();
    order.sort_by(|left, right| scores[*left].total_cmp(&scores[*right]));

    let sorted_names: Vec<&String> = order.iter().map(|index| &names[*index]).collect();
    let totals: Vec<f64> = order.iter().map(|index| totals[*index]).collect();
    let counts: Vec<usize> = order.iter().map(|index| counts[*index]).collect();
    let averages: Vec<f64> = order.iter().map(|index| averages[*index]).collect();
    let alipay: std::collections::HashSet<&str> = rows
        .iter()
        .filter(|row| row.source == "支付宝")
        .map(|row| row.category.as_str())
        .collect();
    let wechat: std::collections::HashSet<&str> = rows
        .iter()
        .filter(|row| row.source == "微信")
        .map(|row| row.category.as_str())
        .collect();

    let alipay_cats: Vec<&&String> = sorted_names
        .iter()
        .filter(|name| alipay.contains(name.as_str()))
        .collect();
    let wechat_cats: Vec<&&String> = sorted_names
        .iter()
        .filter(|name| wechat.contains(name.as_str()) && !alipay.contains(name.as_str()))
        .collect();

    json!({
        "categories": sorted_names,
        "category_groups": {
            "alipay": alipay_cats,
            "wechat": wechat_cats,
        },
        "stats": {
            "totals": totals,
            "counts": counts,
            "averages": averages,
        }
    })
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn counts_as_f64(counts: &[usize]) -> Vec<f64> {
    counts.iter().map(|value| *value as f64).collect()
}

/// pandas `rank(ascending=False)` 默认的 average 规则。
fn average_ranks(values: &[f64], descending: bool) -> Vec<f64> {
    let mut order: Vec<usize> = (0..values.len()).collect();
    order.sort_by(|left, right| {
        if descending {
            values[*right].total_cmp(&values[*left])
        } else {
            values[*left].total_cmp(&values[*right])
        }
    });

    let mut ranks = vec![0.0; values.len()];
    let mut index = 0;
    while index < order.len() {
        let mut end = index + 1;
        while end < order.len() && values[order[end]] == values[order[index]] {
            end += 1;
        }
        let average = ((index + 1) as f64 + end as f64) / 2.0;
        for position in index..end {
            ranks[order[position]] = average;
        }
        index = end;
    }
    ranks
}
