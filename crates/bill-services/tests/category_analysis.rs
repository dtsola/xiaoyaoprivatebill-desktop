//! P2-6 契约测试：分类分析（列表分支）。

use bill_core::{Dataset, Transaction};
use bill_services::category::{category_analysis, category_detail, category_trend};
use chrono::NaiveDate;
use serde_json::json;

fn tx(
    year: i32,
    month: u32,
    direction: &str,
    amount: f64,
    category: &str,
    source: &str,
) -> Transaction {
    let time = NaiveDate::from_ymd_opt(year, month, 10)
        .expect("日期")
        .and_hms_opt(12, 0, 0)
        .expect("时间");
    Transaction {
        time,
        month: format!("{year}-{month:02}"),
        date: format!("{year}-{month:02}-10"),
        direction: direction.into(),
        amount,
        category: category.into(),
        description: String::new(),
        counterparty: String::new(),
        payment_method: String::new(),
        status: "交易成功".into(),
        is_refund: false,
        source: source.into(),
    }
}

fn dataset() -> Dataset {
    Dataset {
        transactions: vec![
            tx(2024, 1, "支出", 100.0, "餐饮", "支付宝"),
            tx(2024, 2, "支出", 50.0, "餐饮", "微信"),
            tx(2024, 2, "支出", 80.0, "交通", "微信"),
            tx(2024, 3, "收入", 300.0, "工资", "支付宝"),
            tx(2023, 5, "支出", 999.0, "餐饮", "支付宝"),
        ],
        files: Vec::new(),
    }
}

#[test]
fn lists_categories_ranked_by_score() {
    let response = category_analysis(&dataset(), None, Some(2024));

    // 餐饮：总额 150、笔数 2，两项排名都第 1；交通：80、1 笔，排名第 2。
    assert_eq!(response["categories"], json!(["餐饮", "交通"]));
    assert_eq!(response["stats"]["totals"], json!([150.0, 80.0]));
    assert_eq!(response["stats"]["counts"], json!([2, 1]));
    assert_eq!(response["stats"]["averages"], json!([75.0, 80.0]));
}

#[test]
fn splits_categories_by_source_without_duplicates() {
    let response = category_analysis(&dataset(), None, Some(2024));
    let groups = &response["category_groups"];

    // 微信独有的分类才进 wechat 组，避免重复展示。
    assert_eq!(groups["alipay"], json!(["餐饮"]));
    assert_eq!(groups["wechat"], json!(["交通"]));
}

#[test]
fn year_filter_limits_rows() {
    let response = category_analysis(&dataset(), None, Some(2023));
    assert_eq!(response["categories"], json!(["餐饮"]));
    assert_eq!(response["stats"]["totals"], json!([999.0]));
}

#[test]
fn detail_branch_returns_stats_trend_pattern_distribution() {
    let response = category_analysis(&dataset(), Some("餐饮"), Some(2024));

    assert_eq!(response["category"], json!("餐饮"));
    assert_eq!(response["stats"]["total_expense"], json!(150.0));
    assert_eq!(response["stats"]["transaction_count"], json!(2));
    assert_eq!(response["stats"]["avg_amount"], json!(75.0));
    assert_eq!(response["stats"]["expense_ratio"], json!(65.22));
    assert_eq!(response["stats"]["max_amount"], json!(100.0));
    assert_eq!(response["stats"]["min_amount"], json!(50.0));
    assert_eq!(response["stats"]["median_amount"], json!(75.0));

    let trend = &response["trend"];
    assert_eq!(trend["dates"].as_array().expect("月份").len(), 12);
    assert_eq!(trend["amounts"][0], json!(100.0));
    assert_eq!(trend["amounts"][1], json!(50.0));

    let pattern = &response["pattern"];
    assert_eq!(pattern["hours"].as_array().expect("小时").len(), 24);
    assert_eq!(pattern["counts"][12], json!(2));

    let distribution = &response["distribution"];
    assert_eq!(distribution["ranges"][0], json!("0-50"));
    assert_eq!(
        distribution["counts"],
        json!([1, 1, 0, 0, 0, 0]),
        "pandas cut 为右闭区间"
    );
}

#[test]
fn category_detail_lists_month_rows_sorted_desc() {
    let response = category_detail(&dataset(), "2024-02", "餐饮");
    let items = response.as_array().expect("数组");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["amount"], json!(50.0));
}

#[test]
fn category_trend_computes_months_and_summary() {
    let response = category_trend(&dataset(), "餐饮");

    assert_eq!(response["months"], json!(["2023-05", "2024-01", "2024-02"]));
    assert_eq!(response["total"], json!([999.0, 100.0, 50.0]));
    assert_eq!(response["transactions"], json!([1, 1, 1]));
    assert_eq!(response["mom_rate"], json!([0.0, -89.99, -50.0]));
    assert_eq!(response["percentage"], json!([100.0, 100.0, 38.46]));
    assert_eq!(response["summary"]["max_month"], json!("2023-05"));
    assert_eq!(response["summary"]["min_month"], json!("2024-02"));
    assert_eq!(response["summary"]["total_transactions"], json!(3));
}
