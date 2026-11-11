//! P4-4 契约测试：10 个只读工具。

use std::sync::Arc;

use bill_api::Runtime;
use bill_core::{DataPaths, Dataset, Transaction};
use bill_mcp::audit;
use bill_mcp::tools::{
    self, AuditParams, CategoryParams, MonthlyParams, SearchParams, TimeParams, YearlyParams,
};
use chrono::NaiveDate;
use serde_json::json;

fn tx(date: &str, direction: &str, amount: f64, category: &str) -> Transaction {
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
        counterparty: "商户".into(),
        payment_method: String::new(),
        status: "交易成功".into(),
        is_refund: false,
        source: "支付宝".into(),
    }
}

fn runtime() -> Arc<Runtime> {
    let dir = tempfile::tempdir().expect("临时目录");
    let paths = DataPaths::new(dir.path().to_path_buf());
    paths.ensure_layout().expect("创建布局");
    std::mem::forget(dir);

    let dataset = Dataset {
        transactions: vec![
            tx("2024-01-05", "支出", 100.0, "餐饮美食"),
            tx("2024-01-06", "支出", 50.0, "交通"),
            tx("2024-01-07", "收入", 300.0, "工资"),
            tx("2024-02-10", "支出", 1200.0, "购物"),
        ],
        files: Vec::new(),
    };
    Arc::new(Runtime::with_dataset(paths, dataset))
}

#[test]
fn overview_reports_years_months_and_counts() {
    let runtime = runtime();
    std::fs::write(runtime.paths.bills_dir().join("alipay.csv"), [0u8; 16]).expect("写文件");

    let data = tools::get_overview(&runtime);
    assert_eq!(data["years"], json!([2024]));
    assert_eq!(data["months"], json!(["2024-02", "2024-01"]));
    assert_eq!(data["transaction_count"], json!(4));
    assert_eq!(data["bill_count"], json!(1));
    assert_eq!(data["is_demo"], json!(false));
}

#[test]
fn list_bills_formats_size_and_demo_flag() {
    let runtime = runtime();
    std::fs::write(
        runtime.paths.bills_dir().join("alipay.csv"),
        vec![0u8; 2048],
    )
    .expect("写文件");

    let data = tools::list_bills(&runtime);
    let files = data["files"].as_array().expect("文件数组");
    assert_eq!(files.len(), 1);
    assert_eq!(files[0]["name"], json!("alipay.csv"));
    assert_eq!(files[0]["size"], json!(2048));
    assert_eq!(files[0]["size_formatted"], json!("2.0 KB"));
    assert_eq!(data["demo"], json!(false));
}

#[test]
fn summary_and_yearly_analysis_match_api() {
    let runtime = runtime();

    let summary = tools::get_summary(&runtime);
    assert_eq!(summary["total_expense"], json!(1350.0));
    assert_eq!(summary["total_income"], json!(300.0));

    let yearly = tools::get_yearly_analysis(
        &runtime,
        &YearlyParams {
            year: Some(2024),
            min_amount: None,
            max_amount: None,
        },
    );
    assert_eq!(yearly["yearly_stats"]["total_expense"], json!(1350.0));
    assert_eq!(
        yearly["trends"]["months"].as_array().expect("月度").len(),
        12
    );
}

#[test]
fn monthly_analysis_returns_daily_series() {
    let runtime = runtime();
    let data = tools::get_monthly_analysis(
        &runtime,
        &MonthlyParams {
            year: 2024,
            month: 1,
            min_amount: None,
            max_amount: None,
        },
    );

    assert_eq!(data["stats"]["total_expense"], json!(150.0));
    assert_eq!(
        data["daily_data"]["expense"]["dates"]
            .as_array()
            .expect("日期")
            .len(),
        31
    );
}

#[test]
fn category_and_time_analysis_return_api_shapes() {
    let runtime = runtime();

    let categories = tools::get_category_analysis(
        &runtime,
        &CategoryParams {
            category: None,
            year: Some(2024),
        },
    );
    assert_eq!(
        categories["categories"],
        json!(["购物", "餐饮美食", "交通"])
    );

    let time = tools::get_time_analysis(
        &runtime,
        &TimeParams {
            year: Some(2024),
            filter: None,
        },
    );
    assert_eq!(
        time["hourly"]["amounts"].as_array().expect("小时").len(),
        24
    );
}

#[test]
fn search_transactions_paginates() {
    let runtime = runtime();
    let data = tools::search_transactions(
        &runtime,
        &SearchParams {
            r#type: Some("支出".into()),
            per_page: Some(2),
            ..SearchParams::default()
        },
    );

    assert_eq!(data["transactions"].as_array().expect("列表").len(), 2);
    assert_eq!(data["pagination"]["total_records"], json!(3));
    assert_eq!(data["summary"]["total_expense"], json!(1350.0));
}

#[test]
fn insights_returns_all_23_keys() {
    let runtime = runtime();
    let data = tools::get_insights(&runtime, &YearlyParams::default());

    assert_eq!(data.as_object().expect("对象").len(), 23);
    assert!(data.get("story_data").is_some());
}

#[test]
fn audit_log_reads_filters_and_limits() {
    let runtime = runtime();
    let paths = &runtime.paths;

    let empty = tools::get_audit_log(paths, &AuditParams::default());
    assert_eq!(empty, json!({ "entries": [] }));

    audit::append(
        paths,
        &json!({"ts": "2026-11-11T10:00:00+08:00", "actor": {"tool": "import_bill"}}),
    )
    .expect("写审计");
    audit::append(
        paths,
        &json!({"ts": "2026-11-11T11:00:00+08:00", "actor": {"tool": "clear_data"}}),
    )
    .expect("写审计");

    let all = tools::get_audit_log(paths, &AuditParams::default());
    assert_eq!(all["entries"].as_array().expect("数组").len(), 2);

    let filtered = tools::get_audit_log(
        paths,
        &AuditParams {
            tool: Some("clear_data".into()),
            ..AuditParams::default()
        },
    );
    assert_eq!(filtered["entries"].as_array().expect("数组").len(), 1);
    assert_eq!(filtered["entries"][0]["actor"]["tool"], json!("clear_data"));

    let limited = tools::get_audit_log(
        paths,
        &AuditParams {
            limit: Some(1),
            ..AuditParams::default()
        },
    );
    assert_eq!(limited["entries"].as_array().expect("数组").len(), 1);
    assert_eq!(limited["entries"][0]["actor"]["tool"], json!("clear_data"));
}
