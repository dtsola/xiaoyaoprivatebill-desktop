//! P2-4 契约测试：年度分析、可用日期、分类。

use bill_api::{handle, ApiRequest, Runtime};
use bill_core::{DataPaths, Dataset, Transaction};
use chrono::NaiveDate;
use serde_json::{json, Value};

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

fn runtime() -> (tempfile::TempDir, Runtime) {
    let dir = tempfile::tempdir().expect("临时目录");
    let paths = DataPaths::new(dir.path().to_path_buf());
    paths.ensure_layout().expect("布局");
    let dataset = Dataset {
        transactions: vec![
            tx(2024, 1, "支出", 100.0, "餐饮"),
            tx(2024, 2, "收入", 300.0, "工资"),
            tx(2023, 5, "支出", 80.0, "交通"),
        ],
        files: Vec::new(),
    };
    (dir, Runtime::with_dataset(paths, dataset))
}

fn call(runtime: &Runtime, path: &str, query: Value) -> Value {
    let req: ApiRequest = serde_json::from_value(json!({
        "method": "GET",
        "path": path,
        "query": query,
    }))
    .expect("构造请求");
    handle(runtime, &req).expect("处理成功")
}

#[test]
fn yearly_analysis_returns_data_envelope() {
    let (_dir, runtime) = runtime();
    let response = call(&runtime, "/yearly_analysis", json!({"year": "2024"}));

    assert_eq!(response["success"], true);
    assert_eq!(
        response["data"]["yearly_stats"]["total_expense"],
        json!(100.0)
    );
    assert_eq!(
        response["data"]["yearly_stats"]["total_income"],
        json!(300.0)
    );
    assert_eq!(
        response["data"]["trends"]["months"]
            .as_array()
            .expect("月份")
            .len(),
        12
    );
}

#[test]
fn yearly_analysis_respects_year_query() {
    let (_dir, runtime) = runtime();
    let response = call(&runtime, "/yearly_analysis", json!({"year": "2023"}));

    assert_eq!(
        response["data"]["yearly_stats"]["total_expense"],
        json!(80.0)
    );
    assert_eq!(response["data"]["yearly_stats"]["total_income"], json!(0.0));
}

#[test]
fn available_dates_lists_months_desc() {
    let (_dir, runtime) = runtime();
    let response = call(&runtime, "/available_dates", json!({}));

    assert_eq!(response["success"], true);
    assert_eq!(response["months"], json!(["2024-02", "2024-01", "2023-05"]));
    assert_eq!(response["years"], json!([2024, 2023]));
}

#[test]
fn categories_are_sorted_unique() {
    let (_dir, runtime) = runtime();
    let response = call(&runtime, "/categories", json!({}));

    assert_eq!(response["success"], true);
    assert_eq!(response["categories"], json!(["交通", "工资", "餐饮"]));
}

#[test]
fn monthly_analysis_requires_year_and_month() {
    let (_dir, runtime) = runtime();
    let missing = call(&runtime, "/monthly_analysis", json!({}));
    assert_eq!(missing["success"], false);

    let ok = call(
        &runtime,
        "/monthly_analysis",
        json!({"year": "2024", "month": "1"}),
    );
    assert_eq!(ok["success"], true);
    assert_eq!(
        ok["data"]["daily_data"]["expense"]["dates"]
            .as_array()
            .expect("日期")
            .len(),
        31
    );
}
