//! P2-3 契约测试：/transactions 外层。

use bill_api::{handle, ApiRequest, Runtime};
use bill_core::{DataPaths, Dataset, Transaction};
use chrono::NaiveDate;
use serde_json::{json, Value};

fn runtime() -> (tempfile::TempDir, Runtime) {
    let dir = tempfile::tempdir().expect("临时目录");
    let paths = DataPaths::new(dir.path().to_path_buf());
    paths.ensure_layout().expect("布局");
    let time = NaiveDate::from_ymd_opt(2024, 1, 10)
        .expect("日期")
        .and_hms_opt(9, 0, 0)
        .expect("时间");
    let dataset = Dataset {
        transactions: vec![Transaction {
            time,
            month: "2024-01".into(),
            date: "2024-01-10".into(),
            direction: "支出".into(),
            amount: 30.0,
            category: "餐饮".into(),
            description: "咖啡".into(),
            counterparty: "星巴克".into(),
            payment_method: String::new(),
            status: "交易成功".into(),
            is_refund: false,
            source: "支付宝".into(),
        }],
        files: Vec::new(),
    };
    (dir, Runtime::with_dataset(paths, dataset))
}

fn call(runtime: &Runtime, query: Value) -> Value {
    let req: ApiRequest = serde_json::from_value(json!({
        "method": "GET",
        "path": "/transactions",
        "query": query,
    }))
    .expect("请求");
    handle(runtime, &req).expect("处理")
}

#[test]
fn transactions_envelope_and_pagination() {
    let (_dir, runtime) = runtime();
    let response = call(&runtime, json!({"page": "1", "per_page": "10"}));

    assert_eq!(response["success"], true);
    assert_eq!(response["pagination"]["current_page"], json!(1));
    assert_eq!(response["transactions"].as_array().expect("列表").len(), 1);
    assert_eq!(response["summary"]["total_expense"], json!(30.0));
}

#[test]
fn transactions_filters_by_year() {
    let (_dir, runtime) = runtime();
    let response = call(&runtime, json!({"year": "2023"}));
    assert_eq!(response["pagination"]["total_records"], json!(0));
}
