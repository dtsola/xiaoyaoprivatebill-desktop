//! P2-8 契约测试：`/analysis` 总装与筛选。

use bill_api::{handle, ApiRequest, Runtime};
use bill_core::{DataPaths, Dataset, Transaction};
use serde_json::{json, Value};

fn tx(date: &str, hour: u32, amount: f64, merchant: &str, category: &str) -> Transaction {
    let time = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .expect("日期")
        .and_hms_opt(hour, 0, 0)
        .expect("时间");
    Transaction {
        time,
        month: time.format("%Y-%m").to_string(),
        date: date.to_string(),
        direction: "支出".into(),
        amount,
        category: category.into(),
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
            tx("2024-01-05", 9, 20.0, "咖啡店", "餐饮美食"),
            tx("2024-01-06", 9, 20.0, "咖啡店", "餐饮美食"),
            tx("2024-02-10", 23, 100.0, "深夜食堂", "餐饮美食"),
            tx("2024-03-01", 8, 200.0, "健身房", "娱乐"),
            tx("2023-05-20", 15, 300.0, "家电店", "购物"),
        ],
        files: Vec::new(),
    }
}

fn runtime_with(dataset: Dataset) -> Runtime {
    let dir = tempfile::tempdir().expect("临时目录");
    let paths = DataPaths::new(dir.path().to_path_buf());
    paths.ensure_layout().expect("创建布局");
    // 目录随测试进程结束回收，但 Dataset 已驻留内存。
    Runtime::with_dataset(paths, dataset)
}

fn request(query: Value) -> ApiRequest {
    serde_json::from_value(json!({
        "method": "GET",
        "path": "/analysis",
        "query": query,
    }))
    .expect("构造请求")
}

const KEYS: [&str; 23] = [
    "merchant_analysis",
    "scenario_analysis",
    "habit_analysis",
    "latte_factor",
    "nighttime_analysis",
    "subscription_analysis",
    "inflation_analysis",
    "brand_loyalty",
    "sankey_data",
    "engel_coefficient",
    "weekend_monday",
    "story_data",
    "tags",
    "payment_analysis",
    "chord_data",
    "funnel_data",
    "quadrant_data",
    "radar_data",
    "wordcloud_data",
    "themeriver_data",
    "boxplot_data",
    "heatmap_data",
    "pareto_data",
];

#[test]
fn analysis_returns_success_and_all_23_keys() {
    let runtime = runtime_with(dataset());
    let response = handle(&runtime, &request(json!({}))).expect("处理成功");

    assert_eq!(response["success"], true);
    let data = response["data"].as_object().expect("data 对象");
    assert_eq!(data.len(), 23, "23 个键，不多不少");
    for key in KEYS {
        assert!(data.contains_key(key), "缺少键 {key}");
    }
}

#[test]
fn analysis_filters_by_year() {
    let runtime = runtime_with(dataset());
    let response = handle(&runtime, &request(json!({"year": "2024"}))).expect("处理成功");
    let data = &response["data"];

    assert_eq!(data["story_data"]["summary"]["tx_count"], json!(4));
    assert_eq!(data["story_data"]["summary"]["total_amount"], json!(340.0));
    assert_eq!(data["engel_coefficient"]["total_expense"], json!(340.0));
}

#[test]
fn analysis_amount_filters_are_truthy_and_max_is_exclusive() {
    let runtime = runtime_with(dataset());
    let response = handle(
        &runtime,
        &request(json!({"min_amount": "20", "max_amount": "200"})),
    )
    .expect("处理成功");

    // min 为大于等于、max 为严格小于：只剩 20、20、100。
    assert_eq!(
        response["data"]["story_data"]["summary"]["tx_count"],
        json!(3)
    );
    assert_eq!(
        response["data"]["story_data"]["summary"]["total_amount"],
        json!(140.0)
    );
}

#[test]
fn analysis_handles_empty_dataset() {
    let runtime = runtime_with(Dataset {
        transactions: Vec::new(),
        files: Vec::new(),
    });
    let response = handle(&runtime, &request(json!({}))).expect("处理成功");

    assert_eq!(response["success"], true);
    assert!(
        response["data"]["story_data"].is_null(),
        "无支出时 story_data 为 null"
    );
    assert_eq!(
        response["data"]["tags"]["spending_power"],
        json!("日均消费0元，属于理性消费人群")
    );
}
