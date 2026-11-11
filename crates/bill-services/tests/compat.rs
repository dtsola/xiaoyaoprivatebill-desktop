//! P2b 契约测试：3 个兼容接口（`/yearly_data`、`/monthly_data`、
//! `/filtered_monthly_analysis`）。
//!
//! 期望值由 Web 版路由体在同一份 11 行数据上运行后取得。

use bill_core::{Dataset, Transaction};
use bill_services::analysis::{filtered_monthly_analysis, monthly_data, yearly_data};
use chrono::NaiveDate;
use serde_json::json;

#[allow(clippy::too_many_arguments)]
fn tx(
    date: &str,
    category: &str,
    merchant: &str,
    direction: &str,
    amount: f64,
    refund: bool,
) -> Transaction {
    let time = NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .expect("日期")
        .and_hms_opt(12, 0, 0)
        .expect("时间");
    Transaction {
        time,
        month: time.format("%Y-%m").to_string(),
        date: date.to_string(),
        direction: direction.into(),
        amount,
        category: category.into(),
        description: String::new(),
        counterparty: merchant.into(),
        payment_method: String::new(),
        status: "交易成功".into(),
        is_refund: refund,
        source: "支付宝".into(),
    }
}

fn dataset() -> Dataset {
    Dataset {
        transactions: vec![
            tx("2023-11-10", "餐饮美食", "A店", "支出", 40.0, false),
            tx("2023-12-10", "交通", "B车", "支出", 80.0, false),
            tx("2023-12-11", "工资", "公司", "收入", 100.0, false),
            tx("2024-01-05", "餐饮美食", "A店", "支出", 100.0, false),
            tx("2024-01-06", "交通", "B车", "支出", 50.0, false),
            tx("2024-01-07", "工资", "公司", "收入", 300.0, false),
            tx("2024-01-08", "餐饮美食", "A店", "支出", 999.0, true),
            tx("2024-02-10", "购物", "C城", "支出", 1200.0, false),
            tx("2024-02-11", "餐饮美食", "A店", "支出", 30.0, false),
            tx("2024-03-10", "购物", "C城", "支出", 2000.0, false),
            tx("2024-03-11", "工资", "公司", "收入", 500.0, false),
        ],
        files: Vec::new(),
    }
}

fn empty() -> Dataset {
    Dataset {
        transactions: Vec::new(),
        files: Vec::new(),
    }
}

#[test]
fn yearly_data_matches_web_reference() {
    let data = yearly_data(&dataset(), Some(2024), "all");

    assert_eq!(
        data["yearly_stats"],
        json!({
            "balance": -2580.0,
            "total_expense": 3380.0,
            "total_income": 800.0,
            "expense_count": 5,
            "income_count": 2,
            "total_count": 7,
            "active_days": 8,
            "avg_transaction": 676.0,
            "avg_daily_expense": 422.5,
            "expense_ratio": 422.5,
            "balance_change": -2560.0,
            "expense_change": 3260.0,
            "income_change": 700.0,
            "transaction_change": 4,
            "balance_change_rate": -12800.0,
            "expense_change_rate": 2716.67,
            "income_change_rate": 700.0,
            "transaction_change_rate": 133.33,
        })
    );
    assert_eq!(data["months"], json!(["2024-01", "2024-02", "2024-03"]));
    assert_eq!(data["expenses"], json!([150.0, 1230.0, 2000.0]));
    assert_eq!(data["incomes"], json!([300.0, 0.0, 500.0]));
    assert_eq!(data["categories"], json!(["购物", "餐饮美食", "交通"]));
    assert_eq!(data["amounts_by_category"], json!([3200.0, 130.0, 50.0]));
    assert_eq!(data["available_years"], json!([2024, 2023]));
    assert_eq!(data["current_year"], json!(2024));
}

#[test]
fn yearly_data_filters_and_missing_last_year() {
    // 上一年被 large 过滤为空 → 8 个变化字段全为 null。
    let large = yearly_data(&dataset(), Some(2024), "large");
    assert_eq!(large["yearly_stats"]["total_expense"], json!(3200.0));
    assert_eq!(large["yearly_stats"]["total_income"], json!(0));
    assert!(large["yearly_stats"]["balance_change"].is_null());
    assert!(large["yearly_stats"]["transaction_change_rate"].is_null());
    assert_eq!(large["months"], json!(["2024-02", "2024-03"]));

    // 2023 的上一年没有任何数据。
    let previous = yearly_data(&dataset(), Some(2023), "all");
    assert_eq!(previous["yearly_stats"]["total_expense"], json!(120.0));
    assert!(previous["yearly_stats"]["expense_change"].is_null());

    // 不传 year 时默认最新年份。
    let default = yearly_data(&dataset(), None, "all");
    assert_eq!(default["current_year"], json!(2024));
    assert_eq!(
        default["yearly_stats"],
        yearly_data(&dataset(), Some(2024), "all")["yearly_stats"]
    );
}

#[test]
fn yearly_data_without_rows_returns_error() {
    let data = yearly_data(&empty(), None, "all");
    assert_eq!(data, json!({"error": "没有可用的账单数据"}));
}

#[test]
fn monthly_data_matches_web_reference() {
    let data = monthly_data(&dataset(), Some(2024), Some(1), "all");

    assert_eq!(data["success"], true);
    assert_eq!(
        data["data"],
        json!({
            "balance": 150.0,
            "total_expense": 150.0,
            "total_income": 300.0,
            "expense_count": 2,
            "income_count": 1,
            "total_count": 3,
            "active_days": 4,
            "avg_transaction": 75.0,
            "avg_daily_expense": 37.5,
            "expense_ratio": 50.0,
            "comparisons": {
                "total_expense": {"diff": 70.0, "rate": 87.5},
                "total_income": {"diff": 200.0, "rate": 200.0},
                "balance": {"diff": 130.0, "rate": 650.0},
            },
        })
    );
}

#[test]
fn monthly_data_omits_comparisons_when_last_month_empty() {
    let data = monthly_data(&dataset(), Some(2024), Some(2), "large");
    assert_eq!(data["data"]["total_expense"], json!(1200.0));
    assert!(data["data"].get("comparisons").is_none());

    let no_last = monthly_data(&dataset(), Some(2023), Some(11), "all");
    assert!(no_last["data"].get("comparisons").is_none());
}

#[test]
fn monthly_data_defaults_to_latest_month_and_keeps_unrounded_rates() {
    let data = monthly_data(&dataset(), None, None, "all");
    assert_eq!(data["data"]["total_expense"], json!(2000.0));
    assert_eq!(
        data["data"]["comparisons"]["total_expense"]["rate"],
        json!(62.601626016260155)
    );
    assert_eq!(
        data["data"]["comparisons"]["total_income"]["rate"],
        json!(0)
    );
    assert_eq!(
        data["data"]["comparisons"]["balance"]["rate"],
        json!(-21.951219512195124)
    );
}

#[test]
fn monthly_data_without_rows_returns_error() {
    let data = monthly_data(&empty(), None, None, "all");
    assert_eq!(data["success"], false);
    assert_eq!(data["error"], json!("没有可用的账单数据"));
}

#[test]
fn filtered_monthly_analysis_matches_web_reference() {
    let data = filtered_monthly_analysis(&dataset(), "all");
    assert_eq!(
        data,
        json!({
            "months": ["2023-11", "2023-12", "2024-01", "2024-02", "2024-03"],
            "total_expenses": [40.0, 80.0, 150.0, 1230.0, 2000.0],
            "transaction_counts": [1, 1, 2, 2, 1],
            "daily_averages": [40.0, 80.0, 75.0, 615.0, 2000.0],
            "mom_rates": [0.0, 100.0, 87.5, 720.0, 62.6],
            "moving_averages": [40.0, 60.0, 90.0, 486.67, 1126.67],
            "categories": ["交通", "购物", "餐饮美食"],
            "category_expenses": [
                [0.0, 0.0, 40.0],
                [80.0, 0.0, 0.0],
                [50.0, 0.0, 100.0],
                [0.0, 1200.0, 30.0],
                [0.0, 2000.0, 0.0],
            ],
        })
    );
}

#[test]
fn filtered_monthly_analysis_filter_boundaries() {
    let large = filtered_monthly_analysis(&dataset(), "large");
    assert_eq!(large["months"], json!(["2024-02", "2024-03"]));
    assert_eq!(large["mom_rates"], json!([0.0, 66.67]));

    // small 是小于等于 1000（与 yearly/monthly 的严格小于不同）。
    let small = filtered_monthly_analysis(&dataset(), "small");
    assert_eq!(
        small["months"],
        json!(["2023-11", "2023-12", "2024-01", "2024-02"])
    );
    assert_eq!(small["total_expenses"], json!([40.0, 80.0, 150.0, 30.0]));
    assert_eq!(small["categories"], json!(["交通", "餐饮美食"]));
}

#[test]
fn filtered_monthly_analysis_without_rows_returns_empty_lists() {
    let data = filtered_monthly_analysis(&empty(), "all");
    assert_eq!(
        data,
        json!({
            "months": [],
            "total_expenses": [],
            "transaction_counts": [],
            "daily_averages": [],
            "mom_rates": [],
            "moving_averages": [],
            "categories": [],
            "category_expenses": [],
        })
    );
}
