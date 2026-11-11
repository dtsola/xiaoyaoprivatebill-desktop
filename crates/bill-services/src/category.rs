//! 分类页基础数据：分类支出与可用年月。
//!
//! 行为基准为 Web 版 `api/analysis.py` 的 `category_expenses`
//! 与 `category_available_dates`。

use std::collections::{BTreeMap, BTreeSet};

use bill_core::Dataset;
use chrono::{Datelike, Timelike};
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
    let all_expense: Vec<&bill_core::Transaction> = dataset
        .transactions
        .iter()
        .filter(|row| row.direction == "支出" && !row.is_refund)
        .collect();

    if let Some(category) = category {
        return category_detail_branch(dataset, &all_expense, category, year);
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

/// 分类明细分支：统计、12 个月趋势、24 小时分布、金额区间分布。
fn category_detail_branch(
    dataset: &Dataset,
    all_expense: &[&bill_core::Transaction],
    category: &str,
    year: Option<i32>,
) -> Value {
    let year = year.or_else(|| dataset.transactions.iter().map(|row| row.time.year()).max());
    let in_year: Vec<&bill_core::Transaction> = all_expense
        .iter()
        .copied()
        .filter(|row| year.is_none_or(|value| row.time.year() == value))
        .collect();
    let rows: Vec<&bill_core::Transaction> = in_year
        .iter()
        .copied()
        .filter(|row| row.category == category)
        .collect();

    if rows.is_empty() {
        return json!({ "error": format!("未找到分类 \"{category}\" 的数据") });
    }

    let total: f64 = rows.iter().map(|row| row.amount).sum();
    let total_all: f64 = in_year.iter().map(|row| row.amount).sum();
    let mut amounts: Vec<f64> = rows.iter().map(|row| row.amount).collect();
    amounts.sort_by(|left, right| left.total_cmp(right));
    let median = if amounts.len().is_multiple_of(2) {
        (amounts[amounts.len() / 2 - 1] + amounts[amounts.len() / 2]) / 2.0
    } else {
        amounts[amounts.len() / 2]
    };

    let prefix = year
        .map(|value| value.to_string())
        .unwrap_or_else(|| "None".into());
    let months: Vec<String> = (1..=12)
        .map(|month| format!("{prefix}-{month:02}"))
        .collect();
    let monthly_total: BTreeMap<&str, f64> =
        in_year.iter().fold(BTreeMap::new(), |mut acc, row| {
            *acc.entry(row.month.as_str()).or_insert(0.0) += row.amount;
            acc
        });
    let category_monthly: BTreeMap<&str, (f64, usize)> =
        rows.iter().fold(BTreeMap::new(), |mut acc, row| {
            let entry = acc.entry(row.month.as_str()).or_insert((0.0, 0));
            entry.0 += row.amount;
            entry.1 += 1;
            acc
        });

    let mut trend_amounts = Vec::new();
    let mut trend_counts = Vec::new();
    let mut trend_ratios = Vec::new();
    for month in &months {
        let (amount, count) = category_monthly
            .get(month.as_str())
            .copied()
            .unwrap_or((0.0, 0));
        let total_month = monthly_total.get(month.as_str()).copied().unwrap_or(0.0);
        trend_amounts.push(round2(amount));
        trend_counts.push(count);
        trend_ratios.push(if total_month > 0.0 {
            round1(amount / total_month * 100.0)
        } else {
            0.0
        });
    }

    let mut hour_counts = vec![0usize; 24];
    let mut hour_sums = [0.0f64; 24];
    for row in &rows {
        let hour = row.time.hour() as usize;
        hour_counts[hour] += 1;
        hour_sums[hour] += row.amount;
    }
    let hour_amounts: Vec<f64> = hour_sums.iter().map(|value| round2(*value)).collect();
    let hour_averages: Vec<f64> = hour_sums
        .iter()
        .zip(&hour_counts)
        .map(|(sum, count)| {
            if *count == 0 {
                0.0
            } else {
                round2(sum / *count as f64)
            }
        })
        .collect();

    let buckets = [0.0, 50.0, 100.0, 200.0, 500.0, 1000.0, f64::INFINITY];
    let labels = ["0-50", "50-100", "100-200", "200-500", "500-1000", "1000+"];
    let mut bucket_counts = vec![0usize; 6];
    // 复刻 pandas cut 的右闭区间：(0,50]、(50,100]……
    for amount in &amounts {
        for index in 0..6 {
            if *amount <= buckets[index + 1] {
                bucket_counts[index] += 1;
                break;
            }
        }
    }
    let percentages: Vec<f64> = bucket_counts
        .iter()
        .map(|count| {
            if rows.is_empty() {
                0.0
            } else {
                round1(*count as f64 / rows.len() as f64 * 100.0)
            }
        })
        .collect();

    json!({
        "category": category,
        "stats": {
            "total_expense": total,
            "transaction_count": rows.len(),
            "avg_amount": round2(total / rows.len() as f64),
            "expense_ratio": if total_all > 0.0 { round2(total / total_all * 100.0) } else { 0.0 },
            "date_range": 365,
            "max_amount": amounts.last().copied().unwrap_or(0.0),
            "min_amount": amounts.first().copied().unwrap_or(0.0),
            "median_amount": median,
        },
        "trend": {
            "dates": months,
            "amounts": trend_amounts,
            "counts": trend_counts,
            "ratios": trend_ratios,
        },
        "pattern": {
            "hours": (0..24).collect::<Vec<i32>>(),
            "counts": hour_counts,
            "amounts": hour_amounts,
            "averages": hour_averages,
        },
        "distribution": {
            "ranges": labels,
            "counts": bucket_counts,
            "percentages": percentages,
        }
    })
}

fn round1(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}
