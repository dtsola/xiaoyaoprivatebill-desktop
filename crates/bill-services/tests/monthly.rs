//! P2-5 契约测试：月度分析。

use bill_core::{Dataset, Transaction};
use bill_services::analysis::{monthly_analysis, MonthlyQuery};
use chrono::NaiveDate;
use serde_json::json;

fn tx(
    year: i32,
    month: u32,
    day: u32,
    direction: &str,
    amount: f64,
    category: &str,
    refund: bool,
) -> Transaction {
    let time = NaiveDate::from_ymd_opt(year, month, day)
        .expect("日期合法")
        .and_hms_opt(12, 0, 0)
        .expect("时间合法");
    Transaction {
        time,
        month: format!("{year}-{month:02}"),
        date: format!("{year}-{month:02}-{day:02}"),
        direction: direction.into(),
        amount,
        category: category.into(),
        description: String::new(),
        counterparty: String::new(),
        payment_method: String::new(),
        status: "交易成功".into(),
        is_refund: refund,
        source: "支付宝".into(),
    }
}

fn dataset() -> Dataset {
    Dataset {
        transactions: vec![
            tx(2024, 1, 5, "支出", 100.0, "餐饮", false),
            tx(2024, 1, 5, "支出", 50.0, "交通", false),
            tx(2024, 1, 20, "收入", 300.0, "工资", false),
            tx(2024, 1, 21, "支出", 999.0, "餐饮", true),
            tx(2023, 12, 10, "支出", 200.0, "餐饮", false),
            tx(2023, 12, 11, "收入", 100.0, "工资", false),
        ],
        files: Vec::new(),
    }
}

fn query() -> MonthlyQuery {
    MonthlyQuery {
        year: 2024,
        month: 1,
        min_amount: None,
        max_amount: None,
    }
}

#[test]
fn computes_stats_and_comparisons() {
    let data = monthly_analysis(&dataset(), &query());
    let stats = &data["stats"];

    assert_eq!(stats["total_expense"], json!(150.0));
    assert_eq!(stats["total_income"], json!(300.0));
    assert_eq!(stats["balance"], json!(150.0));
    assert_eq!(stats["expense_count"], json!(2));
    assert_eq!(stats["income_count"], json!(1));
    assert_eq!(stats["comparisons"]["expense"]["change"], json!(-50.0));
    assert_eq!(stats["comparisons"]["expense"]["rate"], json!(-25.0));
    assert_eq!(stats["comparisons"]["income"]["change"], json!(200.0));
    assert_eq!(stats["comparisons"]["balance"]["change"], json!(250.0));
}

#[test]
fn daily_series_covers_every_day_of_month() {
    let data = monthly_analysis(&dataset(), &query());
    let expense = &data["daily_data"]["expense"];

    let dates = expense["dates"].as_array().expect("日期");
    assert_eq!(dates.len(), 31, "一月应有 31 天");
    assert_eq!(dates[0], json!("2024-01-01"));
    assert_eq!(dates[30], json!("2024-01-31"));

    let amounts = expense["amounts"].as_array().expect("金额");
    assert_eq!(amounts[4], json!(150.0));
    assert_eq!(amounts[5], json!(0.0));
}

#[test]
fn categories_are_split_sorted_and_exclude_refunds() {
    let data = monthly_analysis(&dataset(), &query());

    assert_eq!(
        data["categories"]["expense"]["names"],
        json!(["交通", "餐饮"])
    );
    assert_eq!(
        data["categories"]["expense"]["amounts"],
        json!([50.0, 100.0])
    );
    assert_eq!(data["categories"]["income"]["names"], json!(["工资"]));
}

#[test]
fn january_compares_with_previous_december() {
    let data = monthly_analysis(&dataset(), &query());
    let comparisons = &data["stats"]["comparisons"];

    // 上月为 2023-12：支出 200、收入 100。
    assert_eq!(comparisons["expense"]["change"], json!(-50.0));
    assert_eq!(comparisons["income"]["change"], json!(200.0));
}
