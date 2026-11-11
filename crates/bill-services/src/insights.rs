//! 消费洞察分析（`/analysis` 单元 1–2）。
//!
//! 行为基准为 Web 版 `services/analysis.py`：
//!
//! - 单元 1：`analyze_merchants` / `analyze_scenarios` / `analyze_habits`
//! - 单元 2：`analyze_latte_factor` / `analyze_nighttime_spending` /
//!   `analyze_subscriptions` / `analyze_inflation` / `analyze_brand_loyalty` /
//!   `analyze_engel_coefficient` / `analyze_weekend_vs_monday` /
//!   `analyze_payment_methods`
//!
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

/// 拿铁因子：小额（<30 元）且高频（>5 次）的支出。
pub fn analyze_latte_factor(dataset: &Dataset) -> Value {
    let rows = expenses(dataset);

    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for row in &rows {
        if row.amount < 30.0 {
            *counts.entry(row.counterparty.as_str()).or_insert(0) += 1;
        }
    }

    let frequent: BTreeSet<&str> = counts
        .iter()
        .filter(|(_, count)| **count > 5)
        .map(|(name, _)| *name)
        .collect();

    let mut total_amount = 0.0;
    let mut item_count = 0usize;
    for row in &rows {
        if row.amount < 30.0 && frequent.contains(row.counterparty.as_str()) {
            total_amount += row.amount;
            item_count += 1;
        }
    }

    // pandas `idxmax` 在 groupby 已按商家名排序的序列上取「首个最大值」。
    let mut top_merchant = "未知";
    let mut top_count = 0usize;
    for (name, count) in &counts {
        if *count > top_count {
            top_count = *count;
            top_merchant = name;
        }
    }

    json!({
        "total_amount": total_amount,
        "item_count": item_count,
        "top_merchant": top_merchant,
        "avg_price": if item_count > 0 {
            total_amount / item_count as f64
        } else {
            0.0
        },
    })
}

/// 夜间消费：22:00 - 04:00。
pub fn analyze_nighttime_spending(dataset: &Dataset) -> Value {
    let rows = expenses(dataset);
    let total_spend: f64 = rows.iter().map(|row| row.amount).sum();

    let mut night_total = 0.0;
    let mut night_count = 0usize;
    let mut merchants: BTreeMap<&str, f64> = BTreeMap::new();
    for row in &rows {
        let hour = row.time.hour();
        if hour >= 22 || hour <= 4 {
            night_total += row.amount;
            night_count += 1;
            *merchants.entry(row.counterparty.as_str()).or_insert(0.0) += row.amount;
        }
    }

    let mut top_merchant = "无";
    let mut top_amount = f64::NEG_INFINITY;
    for (name, amount) in &merchants {
        if *amount > top_amount {
            top_amount = *amount;
            top_merchant = name;
        }
    }

    json!({
        "total_amount": night_total,
        "ratio": if total_spend > 0.0 {
            night_total / total_spend * 100.0
        } else {
            0.0
        },
        "top_merchant": top_merchant,
        "transaction_count": night_count,
    })
}

/// 隐形订阅：至少 3 个月、且月扣款标准差小于 5 元。
pub fn analyze_subscriptions(dataset: &Dataset) -> Value {
    let mut monthly: BTreeMap<&str, BTreeMap<&str, f64>> = BTreeMap::new();
    for row in expenses(dataset) {
        *monthly
            .entry(row.counterparty.as_str())
            .or_default()
            .entry(row.month.as_str())
            .or_insert(0.0) += row.amount;
    }

    let mut items: Vec<(&str, f64)> = Vec::new();
    for (name, months) in &monthly {
        if months.len() < 3 {
            continue;
        }
        let count = months.len() as f64;
        let mean = months.values().sum::<f64>() / count;
        // pandas `std` 默认 ddof=1（样本标准差）。
        let variance = months
            .values()
            .map(|value| (value - mean).powi(2))
            .sum::<f64>()
            / (count - 1.0);
        if variance.sqrt() < 5.0 {
            items.push((name, mean));
        }
    }

    // 稳定排序：并列时保持商家名字典序，与 Python `list.sort` 一致。
    items.sort_by(|left, right| (right.1 * 12.0).total_cmp(&(left.1 * 12.0)));

    Value::Array(
        items
            .into_iter()
            .map(|(name, mean)| {
                json!({
                    "name": name,
                    "monthly_amount": mean,
                    "annual_amount": mean * 12.0,
                })
            })
            .collect(),
    )
}

/// 个人通胀率：首末季度客单价变化。
pub fn analyze_inflation(dataset: &Dataset) -> Value {
    let mut quarters: BTreeMap<(i32, u32), (f64, usize)> = BTreeMap::new();
    for row in expenses(dataset) {
        let quarter = (row.time.month() - 1) / 3 + 1;
        let entry = quarters
            .entry((row.time.year(), quarter))
            .or_insert((0.0, 0));
        entry.0 += row.amount;
        entry.1 += 1;
    }

    let averages: Vec<f64> = quarters
        .values()
        .map(|(sum, count)| sum / *count as f64)
        .collect();
    if averages.len() < 2 {
        return json!({ "trend": "stable", "rate": 0 });
    }

    let first_avg = averages[0];
    let last_avg = averages[averages.len() - 1];
    let rate = if first_avg > 0.0 {
        (last_avg - first_avg) / first_avg * 100.0
    } else {
        0.0
    };
    let trend = if rate > 5.0 {
        "up"
    } else if rate < -5.0 {
        "down"
    } else {
        "stable"
    };

    json!({
        "trend": trend,
        "rate": rate,
        "first_avg": first_avg,
        "last_avg": last_avg,
    })
}

/// 品牌忠诚度：金额最高与次数最多的商家。
pub fn analyze_brand_loyalty(dataset: &Dataset) -> Value {
    let rows = expenses(dataset);
    if rows.is_empty() {
        return Value::Null;
    }

    let mut merchants: BTreeMap<&str, (f64, usize)> = BTreeMap::new();
    for row in &rows {
        let entry = merchants
            .entry(row.counterparty.as_str())
            .or_insert((0.0, 0));
        entry.0 += row.amount;
        entry.1 += 1;
    }

    let mut top_amount = ("", 0.0);
    let mut top_count = ("", 0usize);
    for (name, (amount, count)) in &merchants {
        if *amount > top_amount.1 {
            top_amount = (name, *amount);
        }
        if *count > top_count.1 {
            top_count = (name, *count);
        }
    }

    json!({
        "top_amount": { "name": top_amount.0, "value": top_amount.1 },
        "top_count": { "name": top_count.0, "value": top_count.1 },
    })
}

/// 恩格尔系数：`餐饮美食` 分类支出占比。
pub fn analyze_engel_coefficient(dataset: &Dataset) -> Value {
    let rows = expenses(dataset);
    let total_expense: f64 = rows.iter().map(|row| row.amount).sum();
    let food_expense: f64 = rows
        .iter()
        .filter(|row| row.category == "餐饮美食")
        .map(|row| row.amount)
        .sum();
    let coefficient = if total_expense > 0.0 {
        food_expense / total_expense * 100.0
    } else {
        0.0
    };
    let level = if coefficient < 30.0 {
        "富裕"
    } else if coefficient < 40.0 {
        "小康"
    } else if coefficient < 50.0 {
        "温饱"
    } else {
        "贫困"
    };

    json!({
        "coefficient": coefficient,
        "food_expense": food_expense,
        "total_expense": total_expense,
        "level": level,
    })
}

/// 周末与周一消费对比。
pub fn analyze_weekend_vs_monday(dataset: &Dataset) -> Value {
    let mut stats: BTreeMap<&str, (f64, usize)> = BTreeMap::new();
    for row in expenses(dataset) {
        let day = match row.time.weekday() {
            chrono::Weekday::Sat | chrono::Weekday::Sun => "周末",
            chrono::Weekday::Mon => "周一",
            _ => "工作日",
        };
        let entry = stats.entry(day).or_insert((0.0, 0));
        entry.0 += row.amount;
        entry.1 += 1;
    }

    // pandas groupby 按类型名排序，结果顺序为 周一 → 周末 → 工作日。
    const ORDER: [&str; 3] = ["周一", "周末", "工作日"];
    let items: Vec<Value> = ORDER
        .iter()
        .filter_map(|day| {
            stats.get(day).map(|(amount, count)| {
                json!({
                    "day": day,
                    "amount": amount,
                    "count": count,
                    "avg": amount / *count as f64,
                })
            })
        })
        .collect();

    Value::Array(items)
}

/// 支付方式：按账单来源（支付宝/微信）汇总。
pub fn analyze_payment_methods(dataset: &Dataset) -> Value {
    let mut stats: BTreeMap<&str, (f64, usize)> = BTreeMap::new();
    for row in expenses(dataset) {
        let entry = stats.entry(row.source.as_str()).or_insert((0.0, 0));
        entry.0 += row.amount;
        entry.1 += 1;
    }

    let items: Vec<Value> = stats
        .iter()
        .map(|(method, (amount, count))| {
            json!({
                "method": method,
                "amount": round2(*amount),
                "count": count,
            })
        })
        .collect();

    Value::Array(items)
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

    // ===== 单元 2 =====
    // 期望值由 Web 版 `services/analysis.py` 在同一份数据上运行得到。

    fn unit2_dataset() -> Dataset {
        let rows = [
            ("2024-01-05", 9, 20.0, "咖啡店", "餐饮美食", "支付宝"),
            ("2024-01-06", 9, 20.0, "咖啡店", "餐饮美食", "支付宝"),
            ("2024-01-07", 9, 20.0, "咖啡店", "餐饮美食", "支付宝"),
            ("2024-01-08", 9, 20.0, "咖啡店", "餐饮美食", "支付宝"),
            ("2024-01-09", 9, 20.0, "咖啡店", "餐饮美食", "支付宝"),
            ("2024-01-10", 9, 20.0, "咖啡店", "餐饮美食", "支付宝"),
            ("2024-01-11", 10, 25.0, "奶茶店", "餐饮美食", "微信"),
            ("2024-01-12", 10, 25.0, "奶茶店", "餐饮美食", "微信"),
            ("2024-01-13", 11, 10.0, "便利店", "购物", "支付宝"),
            ("2024-01-14", 12, 30.0, "超市", "购物", "微信"),
            ("2024-02-10", 23, 100.0, "深夜食堂", "餐饮美食", "支付宝"),
            ("2024-02-11", 2, 50.0, "凌晨超市", "购物", "微信"),
            ("2024-02-12", 22, 30.0, "便利店", "购物", "微信"),
            ("2024-02-13", 5, 40.0, "早市", "餐饮美食", "支付宝"),
            ("2024-01-01", 8, 15.0, "视频会员", "娱乐", "支付宝"),
            ("2024-02-01", 8, 15.0, "视频会员", "娱乐", "支付宝"),
            ("2024-03-01", 8, 15.0, "视频会员", "娱乐", "支付宝"),
            ("2024-01-05", 8, 10.0, "云存储", "娱乐", "微信"),
            ("2024-02-05", 8, 10.0, "云存储", "娱乐", "微信"),
            ("2024-03-05", 8, 10.0, "云存储", "娱乐", "微信"),
            ("2024-01-10", 8, 200.0, "健身房", "娱乐", "支付宝"),
            ("2024-02-10", 8, 200.0, "健身房", "娱乐", "支付宝"),
            ("2024-04-15", 14, 500.0, "家电店", "购物", "支付宝"),
            ("2024-05-20", 15, 300.0, "服装店", "购物", "微信"),
        ];
        Dataset {
            transactions: rows
                .iter()
                .map(|(date, hour, amount, merchant, category, source)| {
                    let time = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
                        .expect("日期")
                        .and_hms_opt(*hour, 0, 0)
                        .expect("时间");
                    Transaction {
                        time,
                        month: time.format("%Y-%m").to_string(),
                        date: (*date).to_string(),
                        direction: "支出".into(),
                        amount: *amount,
                        category: (*category).into(),
                        description: String::new(),
                        counterparty: (*merchant).into(),
                        payment_method: String::new(),
                        status: "交易成功".into(),
                        is_refund: false,
                        source: (*source).into(),
                    }
                })
                .collect(),
            files: Vec::new(),
        }
    }

    #[test]
    fn latte_factor_keeps_only_frequent_small_expenses() {
        let data = analyze_latte_factor(&unit2_dataset());
        assert_eq!(data["total_amount"], json!(120.0));
        assert_eq!(data["item_count"], json!(6));
        assert_eq!(data["top_merchant"], json!("咖啡店"));
        assert_eq!(data["avg_price"], json!(20.0));
    }

    #[test]
    fn nighttime_covers_22_to_4_clock() {
        let data = analyze_nighttime_spending(&unit2_dataset());
        assert_eq!(data["total_amount"], json!(180.0));
        assert_eq!(data["ratio"], json!(10.557184750733137));
        assert_eq!(data["top_merchant"], json!("深夜食堂"));
        assert_eq!(data["transaction_count"], json!(3));
    }

    #[test]
    fn subscriptions_need_three_months_and_stable_amount() {
        let data = analyze_subscriptions(&unit2_dataset());
        let items = data.as_array().expect("数组");
        assert_eq!(items.len(), 2);
        assert_eq!(items[0]["name"], json!("视频会员"));
        assert_eq!(items[0]["monthly_amount"], json!(15.0));
        assert_eq!(items[0]["annual_amount"], json!(180.0));
        assert_eq!(items[1]["name"], json!("云存储"));
        assert_eq!(items[1]["annual_amount"], json!(120.0));
    }

    #[test]
    fn inflation_compares_first_and_last_quarter() {
        let data = analyze_inflation(&unit2_dataset());
        assert_eq!(data["trend"], json!("up"));
        assert_eq!(data["rate"], json!(872.375690607735));
        assert_eq!(data["first_avg"], json!(41.13636363636363));
        assert_eq!(data["last_avg"], json!(400.0));
    }

    #[test]
    fn inflation_with_single_quarter_is_stable() {
        let mut data = unit2_dataset();
        data.transactions.retain(|row| row.month == "2024-01");
        let result = analyze_inflation(&data);
        assert_eq!(result["trend"], json!("stable"));
        assert_eq!(result["rate"], json!(0));
        assert!(result.get("first_avg").is_none(), "单季度不输出首末均值");
    }

    #[test]
    fn brand_loyalty_picks_top_amount_and_top_count() {
        let data = analyze_brand_loyalty(&unit2_dataset());
        assert_eq!(data["top_amount"]["name"], json!("家电店"));
        assert_eq!(data["top_amount"]["value"], json!(500.0));
        assert_eq!(data["top_count"]["name"], json!("咖啡店"));
        assert_eq!(data["top_count"]["value"], json!(6));
    }

    #[test]
    fn brand_loyalty_is_null_without_expenses() {
        let data = analyze_brand_loyalty(&Dataset {
            transactions: Vec::new(),
            files: Vec::new(),
        });
        assert!(data.is_null());
    }

    #[test]
    fn engel_coefficient_uses_food_category() {
        let data = analyze_engel_coefficient(&unit2_dataset());
        assert_eq!(data["coefficient"], json!(18.181818181818183));
        assert_eq!(data["food_expense"], json!(310.0));
        assert_eq!(data["total_expense"], json!(1705.0));
        assert_eq!(data["level"], json!("富裕"));
    }

    #[test]
    fn weekend_versus_monday_orders_monday_weekend_workday() {
        let data = analyze_weekend_vs_monday(&unit2_dataset());
        let items = data.as_array().expect("数组");
        assert_eq!(items.len(), 3);
        assert_eq!(items[0]["day"], json!("周一"));
        assert_eq!(items[0]["amount"], json!(875.0));
        assert_eq!(items[0]["count"], json!(6));
        assert_eq!(items[0]["avg"], json!(145.83333333333334));
        assert_eq!(items[1]["day"], json!("周末"));
        assert_eq!(items[1]["amount"], json!(430.0));
        assert_eq!(items[1]["count"], json!(7));
        assert_eq!(items[2]["day"], json!("工作日"));
        assert_eq!(items[2]["amount"], json!(400.0));
        assert_eq!(items[2]["count"], json!(11));
        assert_eq!(items[2]["avg"], json!(36.36363636363637));
    }

    #[test]
    fn payment_methods_group_by_source() {
        let data = analyze_payment_methods(&unit2_dataset());
        let items = data.as_array().expect("数组");
        assert_eq!(items.len(), 2);
        assert_eq!(items[0]["method"], json!("微信"));
        assert_eq!(items[0]["amount"], json!(490.0));
        assert_eq!(items[0]["count"], json!(9));
        assert_eq!(items[1]["method"], json!("支付宝"));
        assert_eq!(items[1]["amount"], json!(1215.0));
        assert_eq!(items[1]["count"], json!(15));
    }
}
