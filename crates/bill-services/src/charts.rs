//! 图表数据（`/analysis` 单元 3）。
//!
//! 行为基准为 Web 版：
//!
//! - `services/analysis.py::analyze_sankey`（桑基图）
//! - `services/generators.py` 的 9 个生成器
//!
//! 复刻要点是**顺序**：`unique()` 的首次出现顺序、`groupby` 的键排序、
//! `nlargest` 的并列保序、`pd.cut(observed=False)` 的空桶补零。

use std::collections::{BTreeMap, BTreeSet};

use bill_core::{Dataset, Transaction};
use chrono::{Datelike, Timelike};
use serde_json::{json, Value};

use crate::pycompat::numpy_sum;

/// 星期标签，下标与 `chrono` 的 `num_days_from_monday()` 一致。
const WEEKDAYS: [&str; 7] = ["周一", "周二", "周三", "周四", "周五", "周六", "周日"];

/// 漏斗图金额区间（左闭右开）。
const FUNNEL_LABELS: [&str; 6] = [
    "0-50元",
    "50-100元",
    "100-500元",
    "500-1000元",
    "1000-5000元",
    ">5000元",
];
const FUNNEL_BOUNDS: [f64; 6] = [0.0, 50.0, 100.0, 500.0, 1000.0, 5000.0];

fn expenses(dataset: &Dataset) -> Vec<&Transaction> {
    dataset
        .transactions
        .iter()
        .filter(|row| row.direction == "支出")
        .collect()
}

/// 桑基图：来源 → 分类。
pub fn analyze_sankey(dataset: &Dataset) -> Value {
    let rows = expenses(dataset);

    // `unique()` 保留首次出现顺序，节点下标由它决定。
    let mut sources: Vec<&str> = Vec::new();
    let mut categories: Vec<&str> = Vec::new();
    let mut flows: BTreeMap<(&str, &str), f64> = BTreeMap::new();

    for row in &rows {
        if !sources.contains(&row.source.as_str()) {
            sources.push(row.source.as_str());
        }
        if !categories.contains(&row.category.as_str()) {
            categories.push(row.category.as_str());
        }
        *flows
            .entry((row.category.as_str(), row.source.as_str()))
            .or_insert(0.0) += row.amount;
    }

    let mut nodes = Vec::new();
    for name in &sources {
        nodes.push(json!({ "name": name, "category": "source" }));
    }
    for name in &categories {
        nodes.push(json!({ "name": name, "category": "category" }));
    }

    let links: Vec<Value> = flows
        .iter()
        .filter_map(|((category, source), amount)| {
            let source_index = sources.iter().position(|name| name == source)?;
            let category_index =
                sources.len() + categories.iter().position(|name| name == category)?;
            Some(json!({
                "source": source_index,
                "target": category_index,
                "value": amount,
            }))
        })
        .collect();

    json!({ "nodes": nodes, "links": links })
}

/// 和弦图：星期 → 消费分类。
pub fn generate_chord_data(dataset: &Dataset) -> Value {
    let rows = expenses(dataset);
    if rows.is_empty() {
        return json!({ "nodes": [], "links": [] });
    }

    let sums = category_sums(&rows);
    let top_categories = top_categories(&sums, 10);
    let top_set: BTreeSet<&str> = top_categories.iter().copied().collect();

    let mut flows: BTreeMap<(&str, &str), f64> = BTreeMap::new();
    for row in &rows {
        let category = row.category.as_str();
        if top_set.contains(category) {
            *flows.entry((weekday_label(row), category)).or_insert(0.0) += row.amount;
        }
    }

    let mut nodes: Vec<Value> = WEEKDAYS
        .iter()
        .map(|day| json!({ "name": day, "category": "weekday" }))
        .collect();
    for name in &top_categories {
        nodes.push(json!({ "name": name, "category": "category" }));
    }

    let links: Vec<Value> = flows
        .iter()
        .filter(|(_, amount)| **amount > 0.0)
        .map(|((day, category), amount)| {
            json!({ "source": day, "target": category, "value": amount })
        })
        .collect();

    json!({ "nodes": nodes, "links": links })
}

/// 漏斗图：按金额区间汇总。
pub fn generate_funnel_data(dataset: &Dataset) -> Value {
    let rows = expenses(dataset);
    if rows.is_empty() {
        return Value::Array(Vec::new());
    }

    // `pd.cut(observed=False)` 会保留全部 6 个区间，即使金额为 0。
    let mut sums = [0.0f64; 6];
    for row in &rows {
        if let Some(index) = funnel_bucket(row.amount) {
            sums[index] += row.amount;
        }
    }

    let mut items: Vec<(&str, f64)> = FUNNEL_LABELS.iter().copied().zip(sums).collect();
    items.sort_by(|left, right| right.1.total_cmp(&left.1));

    Value::Array(
        items
            .into_iter()
            .map(|(name, value)| json!({ "name": name, "value": value }))
            .collect(),
    )
}

/// 象限图：频次 vs 均价。
pub fn generate_quadrant_data(dataset: &Dataset) -> Value {
    let rows = expenses(dataset);
    if rows.is_empty() {
        return Value::Array(Vec::new());
    }

    struct Stats<'a> {
        amounts: Vec<f64>,
        categories: BTreeMap<&'a str, usize>,
    }

    let mut merchants: BTreeMap<&str, Stats> = BTreeMap::new();
    for row in &rows {
        let entry = merchants.entry(row.counterparty.as_str()).or_insert(Stats {
            amounts: Vec::new(),
            categories: BTreeMap::new(),
        });
        entry.amounts.push(row.amount);
        *entry.categories.entry(row.category.as_str()).or_insert(0) += 1;
    }

    let items: Vec<Value> = merchants
        .into_iter()
        .filter(|(_, stats)| numpy_sum(&stats.amounts) >= 50.0 && stats.amounts.len() >= 2)
        .map(|(name, stats)| {
            // `mode().iloc[0]`：并列时取字典序最小的分类。
            let mut category = "其他";
            let mut best = 0usize;
            for (candidate, count) in &stats.categories {
                if *count > best {
                    best = *count;
                    category = candidate;
                }
            }
            let total = numpy_sum(&stats.amounts);
            json!({
                "name": name,
                "category": category,
                "frequency": stats.amounts.len(),
                "avg_amount": round2(total / stats.amounts.len() as f64),
                "total_amount": round2(total),
            })
        })
        .collect();

    Value::Array(items)
}

/// 雷达图：季度消费结构。
pub fn generate_radar_data(dataset: &Dataset) -> Value {
    let rows = expenses(dataset);
    if rows.is_empty() {
        return json!({ "indicator": [], "series": [] });
    }

    let sums = category_sums(&rows);
    let top_categories = top_categories(&sums, 8);
    if top_categories.is_empty() {
        return json!({ "indicator": [], "series": [] });
    }

    // 注意：此处只用季度序号（1–4），不区分年份，与 Web 版一致。
    let quarters: BTreeSet<u32> = rows
        .iter()
        .map(|row| (row.time.month() - 1) / 3 + 1)
        .collect();

    let mut series = Vec::new();
    let mut max_value = 0.0f64;
    for quarter in &quarters {
        let values: Vec<f64> = top_categories
            .iter()
            .map(|category| {
                let value: f64 = rows
                    .iter()
                    .filter(|row| {
                        (row.time.month() - 1) / 3 + 1 == *quarter && row.category == *category
                    })
                    .map(|row| row.amount)
                    .sum();
                if value > max_value {
                    max_value = value;
                }
                value
            })
            .collect();
        series.push(json!({ "name": format!("Q{quarter}"), "value": values }));
    }

    let indicator: Vec<Value> = top_categories
        .iter()
        .map(|name| json!({ "name": name, "max": max_value * 1.1 }))
        .collect();

    json!({ "indicator": indicator, "series": series })
}

/// 词云：商户消费金额前 100。
pub fn generate_wordcloud_data(dataset: &Dataset) -> Value {
    let rows = expenses(dataset);
    if rows.is_empty() {
        return Value::Array(Vec::new());
    }

    let sums = category_sums_by_merchant(&rows);
    let mut items: Vec<(&str, f64)> = sums
        .into_iter()
        .filter(|(_, amount)| *amount > 10.0)
        .collect();
    items.sort_by(|left, right| right.1.total_cmp(&left.1));
    items.truncate(100);

    Value::Array(
        items
            .into_iter()
            .map(|(name, value)| json!({ "name": name, "value": value }))
            .collect(),
    )
}

/// 河流图：按月 × 分类。
pub fn generate_themeriver_data(dataset: &Dataset) -> Value {
    let rows = expenses(dataset);
    if rows.is_empty() {
        return json!({ "categories": [], "data": [] });
    }

    let sums = category_sums(&rows);
    let top_categories = top_categories(&sums, 10);

    let months: BTreeSet<&str> = rows.iter().map(|row| row.month.as_str()).collect();
    let mut data = Vec::new();
    for month in &months {
        for category in &top_categories {
            let amount: f64 = rows
                .iter()
                .filter(|row| row.month == *month && row.category == *category)
                .map(|row| row.amount)
                .sum();
            data.push(json!([month, category, amount]));
        }
    }

    json!({ "categories": top_categories, "data": data })
}

/// 箱形图：分类金额分布与散点。
pub fn generate_boxplot_data(dataset: &Dataset) -> Value {
    let rows: Vec<&Transaction> = expenses(dataset)
        .into_iter()
        .filter(|row| row.amount > 0.0)
        .collect();
    if rows.is_empty() {
        return json!({ "categories": [], "data": [] });
    }

    let sums = category_sums(&rows);
    let top_categories = top_categories(&sums, 8);

    let mut points = Vec::new();
    let mut box_data = Vec::new();
    for (index, category) in top_categories.iter().enumerate() {
        let mut values = Vec::new();
        // 散点保持数据集行顺序，与 pandas 逐行遍历一致。
        for row in &rows {
            if row.category == *category {
                values.push(row.amount);
                points.push(json!({
                    "c": index,
                    "v": row.amount,
                    "m": row.counterparty,
                    "d": row.date,
                }));
            }
        }

        if values.is_empty() {
            box_data.push(json!([]));
        } else {
            let mut sorted = values.clone();
            sorted.sort_by(f64::total_cmp);
            box_data.push(json!([
                sorted[0],
                percentile(&sorted, 0.25),
                percentile(&sorted, 0.5),
                percentile(&sorted, 0.75),
                sorted[sorted.len() - 1],
            ]));
        }
    }

    json!({
        "categories": top_categories,
        "data": points,
        "box_data": box_data,
    })
}

/// 热力图：星期 × 小时 的消费笔数。
pub fn generate_heatmap_data(dataset: &Dataset) -> Value {
    let rows = expenses(dataset);
    if rows.is_empty() {
        return Value::Array(Vec::new());
    }

    let mut counts: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for row in &rows {
        let weekday = row.time.weekday().num_days_from_monday();
        *counts.entry((weekday, row.time.hour())).or_insert(0) += 1;
    }

    Value::Array(
        counts
            .into_iter()
            .map(|((weekday, hour), count)| json!([hour, weekday, count]))
            .collect(),
    )
}

/// 帕累托图：分类支出与累计占比。
pub fn generate_pareto_data(dataset: &Dataset) -> Value {
    let rows = expenses(dataset);
    if rows.is_empty() {
        return json!({ "categories": [], "values": [], "percentages": [] });
    }

    let sums = category_sums(&rows);
    let mut items: Vec<(&str, f64)> = sums.into_iter().collect();
    items.sort_by(|left, right| right.1.total_cmp(&left.1));

    let total: f64 = items.iter().map(|(_, value)| *value).sum();
    if total == 0.0 {
        return json!({ "categories": [], "values": [], "percentages": [] });
    }

    let mut cumulative = 0.0;
    let mut categories = Vec::new();
    let mut values = Vec::new();
    let mut percentages = Vec::new();
    for (name, value) in items.iter().take(15) {
        cumulative += value;
        categories.push(json!(name));
        values.push(json!(value));
        percentages.push(json!(round2(cumulative / total * 100.0)));
    }

    json!({
        "categories": categories,
        "values": values,
        "percentages": percentages,
    })
}

fn category_sums<'a>(rows: &[&'a Transaction]) -> BTreeMap<&'a str, f64> {
    let mut sums: BTreeMap<&str, f64> = BTreeMap::new();
    for row in rows {
        *sums.entry(row.category.as_str()).or_insert(0.0) += row.amount;
    }
    sums
}

fn category_sums_by_merchant<'a>(rows: &[&'a Transaction]) -> BTreeMap<&'a str, f64> {
    let mut sums: BTreeMap<&str, f64> = BTreeMap::new();
    for row in rows {
        *sums.entry(row.counterparty.as_str()).or_insert(0.0) += row.amount;
    }
    sums
}

/// `nlargest(n)`：按金额降序，并列时保留键的字典序（`BTreeMap` 已排序）。
fn top_categories<'a>(sums: &BTreeMap<&'a str, f64>, limit: usize) -> Vec<&'a str> {
    let mut items: Vec<(&str, f64)> = sums.iter().map(|(name, value)| (*name, *value)).collect();
    items.sort_by(|left, right| right.1.total_cmp(&left.1));
    items.truncate(limit);
    items.into_iter().map(|(name, _)| name).collect()
}

fn funnel_bucket(amount: f64) -> Option<usize> {
    if amount < 0.0 {
        // `pd.cut` 对负金额给出 NaN，`groupby` 默认丢弃。
        return None;
    }
    (0..FUNNEL_LABELS.len())
        .rev()
        .find(|index| amount >= FUNNEL_BOUNDS[*index])
}

fn weekday_label(row: &Transaction) -> &'static str {
    WEEKDAYS[row.time.weekday().num_days_from_monday() as usize]
}

/// 线性插值分位数，与 `np.percentile` 默认方法一致（不四舍五入）。
fn percentile(sorted: &[f64], p: f64) -> f64 {
    let position = p * (sorted.len() - 1) as f64;
    let lower = position.floor() as usize;
    let upper = position.ceil() as usize;
    if lower == upper {
        sorted[lower]
    } else {
        let weight = position - lower as f64;
        sorted[lower] + (sorted[upper] - sorted[lower]) * weight
    }
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}
