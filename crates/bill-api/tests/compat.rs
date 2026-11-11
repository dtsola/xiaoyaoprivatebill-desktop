//! P2b 契约测试：3 个兼容接口的路由注册与参数处理。

use bill_api::{handle, ApiRequest, Runtime};
use bill_core::{DataPaths, Dataset, Transaction};
use serde_json::{json, Value};

fn tx(date: &str, category: &str, direction: &str, amount: f64, refund: bool) -> Transaction {
    let time = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
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
            tx("2023-12-10", "交通", "支出", 80.0, false),
            tx("2023-12-11", "工资", "收入", 100.0, false),
            tx("2024-01-05", "餐饮美食", "支出", 100.0, false),
            tx("2024-01-06", "餐饮美食", "支出", 50.0, false),
            tx("2024-01-07", "工资", "收入", 300.0, false),
            tx("2024-01-08", "餐饮美食", "支出", 999.0, true),
            tx("2024-02-10", "购物", "支出", 1200.0, false),
        ],
        files: Vec::new(),
    }
}

fn runtime_with(dataset: Dataset) -> Runtime {
    let dir = tempfile::tempdir().expect("临时目录");
    let paths = DataPaths::new(dir.path().to_path_buf());
    paths.ensure_layout().expect("创建布局");
    Runtime::with_dataset(paths, dataset)
}

fn get(path: &str, query: Value) -> ApiRequest {
    serde_json::from_value(json!({
        "method": "GET",
        "path": path,
        "query": query,
    }))
    .expect("构造请求")
}

#[test]
fn yearly_data_route_defaults_to_latest_year() {
    let runtime = runtime_with(dataset());
    let response = handle(&runtime, &get("/yearly_data", json!({}))).expect("处理成功");

    assert_eq!(response["current_year"], json!(2024));
    assert_eq!(response["yearly_stats"]["total_expense"], json!(1350.0));
    assert_eq!(response["months"], json!(["2024-01", "2024-02"]));
    assert_eq!(response["available_years"], json!([2024, 2023]));
}

#[test]
fn yearly_data_route_honours_year_and_filter() {
    let runtime = runtime_with(dataset());
    let response = handle(
        &runtime,
        &get("/yearly_data", json!({"year": "2023", "filter": "large"})),
    )
    .expect("处理成功");

    assert_eq!(response["current_year"], json!(2023));
    assert_eq!(response["yearly_stats"]["total_expense"], json!(0));
    assert_eq!(response["months"], json!([]));
}

#[test]
fn monthly_data_route_defaults_to_latest_month() {
    let runtime = runtime_with(dataset());
    let response = handle(&runtime, &get("/monthly_data", json!({}))).expect("处理成功");

    assert_eq!(response["success"], true);
    assert_eq!(response["data"]["total_expense"], json!(1200.0));
}

#[test]
fn monthly_data_route_returns_comparisons() {
    let runtime = runtime_with(dataset());
    let response = handle(
        &runtime,
        &get("/monthly_data", json!({"year": "2024", "month": "1"})),
    )
    .expect("处理成功");

    assert_eq!(response["data"]["total_expense"], json!(150.0));
    assert_eq!(
        response["data"]["comparisons"]["total_expense"]["diff"],
        json!(70.0)
    );
}

#[test]
fn filtered_monthly_route_passes_filter() {
    let runtime = runtime_with(dataset());
    let all = handle(&runtime, &get("/filtered_monthly_analysis", json!({}))).expect("处理成功");
    assert_eq!(all["months"], json!(["2023-12", "2024-01", "2024-02"]));

    let large = handle(
        &runtime,
        &get("/filtered_monthly_analysis", json!({"filter": "large"})),
    )
    .expect("处理成功");
    assert_eq!(large["months"], json!(["2024-02"]));
}

#[test]
fn compat_routes_handle_empty_dataset() {
    let runtime = runtime_with(Dataset {
        transactions: Vec::new(),
        files: Vec::new(),
    });

    let yearly = handle(&runtime, &get("/yearly_data", json!({}))).expect("处理成功");
    assert_eq!(yearly["error"], json!("没有可用的账单数据"));

    let monthly = handle(&runtime, &get("/monthly_data", json!({}))).expect("处理成功");
    assert_eq!(monthly["success"], false);

    let filtered =
        handle(&runtime, &get("/filtered_monthly_analysis", json!({}))).expect("处理成功");
    assert_eq!(filtered["months"], json!([]));
}
