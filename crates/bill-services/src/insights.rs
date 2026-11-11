//! 消费洞察分析（`/analysis` 单元 1）。
//!
//! 行为基准为 Web 版 `services/analysis.py` 的
//! `analyze_merchants` / `analyze_scenarios` / `analyze_habits`。
//! 本模块只提供函数级能力，路由要等 23 个键齐备后接线。

use std::collections::{BTreeMap, BTreeSet};

use bill_core::{Dataset, Transaction};
use chrono::{Datelike, Timelike};
use serde_json::{json, Value};

/// 线上渠道关键词（与 Web 版逐字一致）。
const ONLINE_KEYWORDS: [&str; 15] = [
    "淘宝",
    "天猫",
    "京东",
    "拼多多",
    "美团",
    "饿了么",
    "App Store",
    "Steam",
    "Apple Music",
    "iCloud",
    "网易",
    "支付宝",
    "微信",
    "闲鱼",
    "得物",
];

fn expenses(dataset: &Dataset) -> Vec<&Transaction> {
    dataset
        .transactions
        .iter()
        .filter(|row| row.direction == "支出")
        .collect()
}

/// 商家消费分析。
pub fn analyze_merchants(dataset: &Dataset) -> Value {
    let rows = expenses(dataset);

    let mut stats: BTreeMap<String, (usize, f64, String, String)> = BTreeMap::new();
    for row in &rows {
        let entry = stats.entry(row.counterparty.clone()).or_insert((
            0,
            0.0,
            row.date.clone(),
            row.date.clone(),
        ));
        entry.0 += 1;
        entry.1 += row.amount;
        if row.date < entry.2 {
            entry.2 = row.date.clone();
        }
        if row.date > entry.3 {
            entry.3 = row.date.clone();
        }
    }

    let mut merchant_stats = serde_json::Map::new();
    let mut frequent = Vec::new();
    let min_count = if dataset.transactions.len() >= 50 {
        2
    } else {
        1
    };

    for (name, (count, total, first, last)) in &stats {
        let span = span_days(first, last);
        merchant_stats.insert(
            name.clone(),
            json!({
                "交易次数": count,
                "总金额": round2(*total),
                "平均金额": round2(total / *count as f64),
                "交易跨度": span,
            }),
        );
        if *count >= min_count {
            frequent.push(json!({
                "name": name,
                "amount": total,
                "count": count,
                "last_visit": last,
            }));
        }
    }

    frequent.sort_by(|left, right| {
        right["amount"]
            .as_f64()
            .unwrap_or_default()
            .total_cmp(&left["amount"].as_f64().unwrap_or_default())
    });
    frequent.truncate(20);

    json!({ "merchant_stats": merchant_stats, "frequent_merchants": frequent })
}

/// 消费场景分析：渠道、时段、金额层级。
pub fn analyze_scenarios(dataset: &Dataset) -> Value {
    let mut grouped: BTreeMap<(String, String), f64> = BTreeMap::new();
    for row in expenses(dataset) {
        let scene = if ONLINE_KEYWORDS
            .iter()
            .any(|keyword| row.counterparty.contains(keyword))
        {
            "线上"
        } else {
            "线下"
        };
        for (category, name) in [
            ("渠道", scene.to_owned()),
            ("时段", period_of(row.time.hour()).to_owned()),
            ("层级", level_of(row.amount).to_owned()),
        ] {
            *grouped.entry((category.to_owned(), name)).or_insert(0.0) += row.amount;
        }
    }

    let items: Vec<Value> = grouped
        .into_iter()
        .map(|((category, name), amount)| json!({ "name": name, "value": amount, "category": category }))
        .collect();
    Value::Array(items)
}

/// 消费习惯分析。
pub fn analyze_habits(dataset: &Dataset) -> Value {
    let rows = expenses(dataset);
    let total: f64 = rows.iter().map(|row| row.amount).sum();

    let mut daily: BTreeMap<String, f64> = BTreeMap::new();
    let mut months: BTreeSet<String> = BTreeSet::new();
    let mut weekend: f64 = 0.0;
    let mut month_start: f64 = 0.0;
    let mut merchant_months: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    for row in &rows {
        *daily.entry(row.date.clone()).or_insert(0.0) += row.amount;
        months.insert(row.month.clone());
        if matches!(
            row.time.weekday(),
            chrono::Weekday::Sat | chrono::Weekday::Sun
        ) {
            weekend += row.amount;
        }
        if row.time.day() <= 5 {
            month_start += row.amount;
        }
        merchant_months
            .entry(row.counterparty.clone())
            .or_default()
            .insert(row.month.clone());
    }

    let daily_avg = if daily.is_empty() {
        0.0
    } else {
        daily.values().sum::<f64>() / daily.len() as f64
    };

    let threshold = (months.len() as f64 * 0.8).max(2.0);
    let fixed: f64 = rows
        .iter()
        .filter(|row| {
            merchant_months
                .get(&row.counterparty)
                .is_some_and(|used| used.len() as f64 >= threshold)
        })
        .map(|row| row.amount)
        .sum();

    json!({
        "daily_avg": round2(daily_avg),
        "active_days": daily.len(),
        "weekend_ratio": if total > 0.0 { round1(weekend / total * 100.0) } else { 0.0 },
        "fixed_expenses": if total > 0.0 { round1(fixed / total * 100.0) } else { 0.0 },
        "month_start_ratio": if total > 0.0 { round1(month_start / total * 100.0) } else { 0.0 },
    })
}

fn period_of(hour: u32) -> &'static str {
    match hour {
        6..=8 => "清晨(6-9点)",
        9..=11 => "上午(9-12点)",
        12..=13 => "中午(12-14点)",
        14..=16 => "下午(14-17点)",
        17..=19 => "傍晚(17-20点)",
        20..=22 => "晚上(20-23点)",
        _ => "深夜(23-6点)",
    }
}

fn level_of(amount: f64) -> &'static str {
    if amount >= 1000.0 {
        "大额(1000+)"
    } else if amount >= 300.0 {
        "中额(300-1000)"
    } else if amount >= 100.0 {
        "小额(100-300)"
    } else {
        "零花(0-100)"
    }
}

fn span_days(first: &str, last: &str) -> i64 {
    let parse = |value: &str| chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").ok();
    match (parse(first), parse(last)) {
        (Some(a), Some(b)) => (b - a).num_days() + 1,
        _ => 1,
    }
}

fn round1(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn tx(date: (i32, u32, u32), hour: u32, amount: f64, merchant: &str) -> Transaction {
        let time = NaiveDate::from_ymd_opt(date.0, date.1, date.2)
            .expect("日期")
            .and_hms_opt(hour, 0, 0)
            .expect("时间");
        Transaction {
            time,
            month: time.format("%Y-%m").to_string(),
            date: time.format("%Y-%m-%d").to_string(),
            direction: "支出".into(),
            amount,
            category: "餐饮".into(),
            description: String::new(),
            counterparty: merchant.into(),
            payment_method: String::new(),
            status: "交易成功".into(),
            is_refund: false,
            source: "支付宝".into(),
        }
    }

    fn dataset() -> Dataset {
        Dataset {
            transactions: vec![
                tx((2024, 1, 6), 8, 100.0, "淘宝"),
                tx((2024, 1, 7), 15, 50.0, "淘宝"),
                tx((2024, 1, 8), 23, 500.0, "星巴克"),
            ],
            files: Vec::new(),
        }
    }

    #[test]
    fn merchants_stats_and_frequency() {
        let data = analyze_merchants(&dataset());
        assert_eq!(data["merchant_stats"]["淘宝"]["交易次数"], json!(2));
        assert_eq!(data["merchant_stats"]["淘宝"]["总金额"], json!(150.0));
        assert_eq!(data["merchant_stats"]["淘宝"]["平均金额"], json!(75.0));
        assert_eq!(data["merchant_stats"]["淘宝"]["交易跨度"], json!(2));

        let frequent = data["frequent_merchants"].as_array().expect("常客");
        assert_eq!(frequent[0]["name"], json!("星巴克"));
        assert_eq!(frequent[0]["count"], json!(1));
    }

    #[test]
    fn scenarios_cover_channel_period_and_level() {
        let data = analyze_scenarios(&dataset());
        let items = data.as_array().expect("数组");
        let find = |category: &str, name: &str| {
            items
                .iter()
                .find(|item| item["category"] == category && item["name"] == name)
                .unwrap_or_else(|| panic!("缺少 {category}/{name}"))
                .clone()
        };

        assert_eq!(find("渠道", "线上")["value"], json!(150.0));
        assert_eq!(find("渠道", "线下")["value"], json!(500.0));
        assert_eq!(find("时段", "清晨(6-9点)")["value"], json!(100.0));
        assert_eq!(find("时段", "深夜(23-6点)")["value"], json!(500.0));
        assert_eq!(find("层级", "中额(300-1000)")["value"], json!(500.0));
        assert_eq!(find("层级", "小额(100-300)")["value"], json!(100.0));
        assert_eq!(find("层级", "零花(0-100)")["value"], json!(50.0));
    }

    #[test]
    fn habits_ratios() {
        let data = analyze_habits(&dataset());
        assert_eq!(data["active_days"], json!(3));
        assert_eq!(data["daily_avg"], json!(216.67));
        assert_eq!(data["weekend_ratio"], json!(23.1));
        assert_eq!(data["month_start_ratio"], json!(0.0));
    }
}
