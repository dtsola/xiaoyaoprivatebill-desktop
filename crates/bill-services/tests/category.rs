//! P2-6 契约测试：分类支出与分类可用日期。

use bill_core::{Dataset, Transaction};
use bill_services::category::{category_available_dates, category_expenses};
use chrono::NaiveDate;
use serde_json::json;

fn tx(year: i32, month: u32, direction: &str, amount: f64, category: &str) -> Transaction {
    let time = NaiveDate::from_ymd_opt(year, month, 10)
        .expect("日期合法")
        .and_hms_opt(12, 0, 0)
        .expect("时间合法");
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
        source: "支付宝".into(),
    }
}

fn dataset() -> Dataset {
    Dataset {
        transactions: vec![
            tx(2024, 1, "支出", 100.0, "餐饮"),
            tx(2024, 2, "支出", 50.0, "交通"),
            tx(2024, 2, "支出", 20.0, "餐饮"),
            tx(2024, 1, "收入", 300.0, "工资"),
            tx(2023, 5, "支出", 80.0, "交通"),
        ],
        files: Vec::new(),
    }
}

#[test]
fn category_expenses_sums_only_expenses_sorted_desc() {
    let response = category_expenses(&dataset());

    assert_eq!(response["categories"], json!(["交通", "餐饮"]));
    assert_eq!(response["amounts"], json!([130.0, 120.0]));
}

#[test]
fn category_available_dates_groups_months_by_year() {
    let response = category_available_dates(&dataset());

    assert_eq!(response["years"], json!([2024, 2023]));
    assert_eq!(response["months"]["2024"], json!([1, 2]));
    assert_eq!(response["months"]["2023"], json!([5]));
}
