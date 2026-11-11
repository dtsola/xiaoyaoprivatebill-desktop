//! P2-3 契约测试：交易记录分页与筛选。

use bill_core::{Dataset, Transaction};
use bill_services::transactions::{list_transactions, TransactionQuery};
use chrono::NaiveDate;
use serde_json::json;

fn tx(
    day: u32,
    hour: u32,
    direction: &str,
    amount: f64,
    desc: &str,
    category: &str,
    refund: bool,
) -> Transaction {
    let time = NaiveDate::from_ymd_opt(2024, 1, day)
        .expect("日期合法")
        .and_hms_opt(hour, 0, 0)
        .expect("时间合法");
    Transaction {
        time,
        month: "2024-01".into(),
        date: format!("2024-01-{day:02}"),
        direction: direction.into(),
        amount,
        category: category.into(),
        description: desc.into(),
        counterparty: "商户".into(),
        payment_method: String::new(),
        status: "交易成功".into(),
        is_refund: refund,
        source: "支付宝".into(),
    }
}

fn dataset() -> Dataset {
    Dataset {
        transactions: vec![
            tx(10, 9, "支出", 30.0, "咖啡", "餐饮", false),
            tx(11, 10, "支出", 80.0, "午餐", "餐饮", false),
            tx(12, 11, "收入", 300.0, "工资", "工资", false),
            tx(13, 12, "支出", 999.0, "退款", "餐饮", true),
        ],
        files: Vec::new(),
    }
}

fn query() -> TransactionQuery {
    TransactionQuery::default()
}

#[test]
fn default_page_is_first_twenty_sorted_desc() {
    let response = list_transactions(&dataset(), &query());

    assert_eq!(response["success"], true);
    let items = response["transactions"].as_array().expect("列表");
    assert_eq!(items.len(), 3, "退款行被排除");
    assert_eq!(items[0]["time"], json!("2024-01-12 11:00:00"), "按时间倒序");
    assert_eq!(response["pagination"]["current_page"], json!(1));
    assert_eq!(response["pagination"]["per_page"], json!(20));
    assert_eq!(response["pagination"]["total_records"], json!(3));
}

#[test]
fn summary_uses_filtered_rows() {
    let response = list_transactions(&dataset(), &query());
    assert_eq!(response["summary"]["total_income"], json!(300.0));
    assert_eq!(response["summary"]["total_expense"], json!(110.0));
    assert_eq!(response["summary"]["balance"], json!(190.0));
}

#[test]
fn pagination_clamps_page() {
    let mut request = query();
    request.per_page = 2;
    request.page = 99;
    let response = list_transactions(&dataset(), &request);

    assert_eq!(response["pagination"]["total_pages"], json!(2));
    assert_eq!(response["pagination"]["current_page"], json!(2));
    let items = response["transactions"].as_array().expect("列表");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["description"], json!("咖啡"));
}

#[test]
fn filters_by_type_category_and_hour() {
    let mut request = query();
    request.r#type = Some("支出".into());
    request.category = Some("餐饮".into());
    request.hour = Some(10);
    let response = list_transactions(&dataset(), &request);

    let items = response["transactions"].as_array().expect("列表");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["description"], json!("午餐"));
}

#[test]
fn search_is_case_insensitive_regex() {
    let mut request = query();
    request.search = Some("咖.".into());
    let response = list_transactions(&dataset(), &request);

    let items = response["transactions"].as_array().expect("列表");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["description"], json!("咖啡"));
}

#[test]
fn invalid_regex_returns_error() {
    let mut request = query();
    request.search = Some("(".into());
    let response = list_transactions(&dataset(), &request);

    assert_eq!(response["success"], false);
    assert!(
        response["error"]
            .as_str()
            .expect("错误信息")
            .contains("获取交易记录失败"),
        "错误信息: {}",
        response["error"]
    );
}

#[test]
fn amount_bounds_are_inclusive_and_zero_ignored() {
    let mut request = query();
    request.min_amount = Some(0.0);
    request.max_amount = Some(30.0);
    let response = list_transactions(&dataset(), &request);

    let items = response["transactions"].as_array().expect("列表");
    assert_eq!(items.len(), 1, "min=0 视为未设置，max 为小于等于");
    assert_eq!(items[0]["amount"], json!(30.0));
}
