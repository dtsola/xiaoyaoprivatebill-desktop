//! P2-4 契约测试：年度分析。

use bill_core::{Dataset, Transaction};
use bill_services::analysis::{yearly_analysis, YearlyQuery};
use chrono::NaiveDate;
use serde_json::json;

fn tx(
    date: (i32, u32, u32),
    direction: &str,
    amount: f64,
    category: &str,
    refund: bool,
) -> Transaction {
    let time = NaiveDate::from_ymd_opt(date.0, date.1, date.2)
        .expect("日期合法")
        .and_hms_opt(12, 0, 0)
        .expect("时间合法");
    Transaction {
        time,
        month: time.format("%Y-%m").to_string(),
        date: time.format("%Y-%m-%d").to_string(),
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
            tx((2024, 1, 10), "支出", 100.0, "餐饮", false),
            tx((2024, 2, 11), "支出", 50.0, "交通", false),
            tx((2024, 3, 12), "支出", 999.0, "餐饮", true),
            tx((2024, 1, 15), "收入", 300.0, "工资", false),
            tx((2023, 5, 1), "支出", 200.0, "餐饮", false),
            tx((2023, 6, 1), "收入", 100.0, "工资", false),
        ],
        files: Vec::new(),
    }
}

#[test]
fn computes_yearly_stats_and_comparisons() {
    let data = yearly_analysis(
        &dataset(),
        &YearlyQuery {
            year: Some(2024),
            min_amount: None,
            max_amount: None,
        },
    );
    let stats = &data["yearly_stats"];

    assert_eq!(stats["total_expense"], json!(150.0));
    assert_eq!(stats["total_income"], json!(300.0));
    assert_eq!(stats["balance"], json!(150.0));
    assert_eq!(stats["expense_count"], json!(2));
    assert_eq!(stats["income_count"], json!(1));
    assert_eq!(stats["total_count"], json!(3));
    // 退款行也计入活跃天数（与 Python 一致）
    assert_eq!(stats["active_days"], json!(4));
    assert_eq!(stats["avg_transaction"], json!(75.0));
    assert_eq!(stats["avg_daily_expense"], json!(37.5));
    assert_eq!(stats["avg_monthly_income"], json!(25.0));
    assert_eq!(stats["expense_ratio"], json!(50.0));

    let comparisons = &stats["comparisons"];
    assert_eq!(comparisons["expense"]["change"], json!(-50.0));
    assert_eq!(comparisons["expense"]["rate"], json!(-25.0));
    assert_eq!(comparisons["income"]["change"], json!(200.0));
    assert_eq!(comparisons["income"]["rate"], json!(200.0));
    assert_eq!(comparisons["balance"]["change"], json!(250.0));
    assert_eq!(comparisons["balance"]["rate"], json!(250.0));
    assert_eq!(comparisons["count"]["change"], json!(1));
    assert_eq!(comparisons["count"]["rate"], json!(50.0));
}

#[test]
fn fills_all_twelve_months() {
    let data = yearly_analysis(
        &dataset(),
        &YearlyQuery {
            year: Some(2024),
            min_amount: None,
            max_amount: None,
        },
    );
    let trends = &data["trends"];

    let months = trends["months"].as_array().expect("months");
    assert_eq!(months.len(), 12);
    assert_eq!(months[0], json!("2024-01"));
    assert_eq!(months[11], json!("2024-12"));

    let expenses = trends["expenses"].as_array().expect("expenses");
    assert_eq!(expenses[0], json!(100.0));
    assert_eq!(expenses[1], json!(50.0));
    assert_eq!(expenses[2], json!(0.0));

    let incomes = trends["incomes"].as_array().expect("incomes");
    assert_eq!(incomes[0], json!(300.0));
}

#[test]
fn categories_are_sorted_and_split_by_source() {
    let data = yearly_analysis(
        &dataset(),
        &YearlyQuery {
            year: Some(2024),
            min_amount: None,
            max_amount: None,
        },
    );

    assert_eq!(
        data["categories"]["expense"]["names"],
        json!(["交通", "餐饮"])
    );
    assert_eq!(
        data["categories"]["expense"]["amounts"],
        json!([50.0, 100.0])
    );
    assert_eq!(data["categories"]["income"]["names"], json!(["工资"]));

    let source = data["categories_source"]["expense"]
        .as_array()
        .expect("来源分类");
    assert_eq!(source.len(), 2);
    assert_eq!(source[0]["来源"], json!("支付宝"));
}

#[test]
fn amount_filters_apply_before_splitting() {
    let data = yearly_analysis(
        &dataset(),
        &YearlyQuery {
            year: Some(2024),
            min_amount: Some(60.0),
            max_amount: None,
        },
    );
    let stats = &data["yearly_stats"];

    assert_eq!(stats["total_expense"], json!(100.0));
    assert_eq!(stats["expense_count"], json!(1));
}

#[test]
fn without_last_year_comparisons_are_null() {
    let data = yearly_analysis(
        &dataset(),
        &YearlyQuery {
            year: Some(2021),
            min_amount: None,
            max_amount: None,
        },
    );
    let comparisons = &data["yearly_stats"]["comparisons"];

    assert_eq!(comparisons["expense"]["change"], serde_json::Value::Null);
    assert_eq!(comparisons["expense"]["rate"], serde_json::Value::Null);
}

#[test]
fn time_analysis_buckets_hours_and_weekend() {
    // 2024-01-10 是周三，2024-01-13 是周六。
    let dataset = Dataset {
        transactions: vec![
            tx((2024, 1, 10), "支出", 30.0, "餐饮", false),
            tx((2024, 1, 13), "支出", 70.0, "餐饮", false),
        ],
        files: Vec::new(),
    };
    let data = bill_services::analysis::time_analysis(&dataset, Some(2024), "all");

    assert_eq!(data["hourly"]["amounts"][12], json!(100.0));
    assert_eq!(data["hourly"]["counts"][12], json!(2));
    assert_eq!(
        data["weekday_weekend"]["餐饮"]["weekday"]["amount"],
        json!(30.0)
    );
    assert_eq!(
        data["weekday_weekend"]["餐饮"]["weekend"]["percentage"],
        json!(70.0)
    );
}
