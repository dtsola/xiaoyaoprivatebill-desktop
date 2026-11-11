//! P1-9 契约测试：golden 对账。
//!
//! 期望值由 `tools/golden/generate.py` 生成，Rust 侧只读不写。

use bill_api::{handle, ApiRequest, Runtime};
use bill_core::DataPaths;
use bill_services::demo_dataset;
use serde_json::Value;

const EXPECTED: &str = include_str!("../../../fixtures/expected/available_years.json");

fn request(path: &str, query: &[(&str, &str)]) -> ApiRequest {
    let query: std::collections::BTreeMap<String, String> = query
        .iter()
        .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
        .collect();
    serde_json::from_value(serde_json::json!({
        "method": "GET",
        "path": path,
        "query": query,
    }))
    .expect("构造请求")
}

fn compare(path: &str, query: &[(&str, &str)], expected: &str) {
    let dir = tempfile::tempdir().expect("临时目录");
    let paths = DataPaths::new(dir.path().to_path_buf());
    paths.ensure_layout().expect("创建布局");
    let runtime = Runtime::with_dataset(paths, demo_dataset().expect("演示数据"));

    let actual = handle(&runtime, &request(path, query)).expect("处理成功");
    let expected: Value = serde_json::from_str(expected).expect("期望 JSON 合法");
    assert_close(&actual, &expected, path);
}

#[test]
fn available_years_matches_python_golden() {
    let dir = tempfile::tempdir().expect("临时目录");
    let paths = DataPaths::new(dir.path().to_path_buf());
    paths.ensure_layout().expect("创建布局");

    let runtime = Runtime::with_dataset(paths, demo_dataset().expect("演示数据"));
    let req: ApiRequest = serde_json::from_value(serde_json::json!({
        "method": "GET",
        "path": "/available_years",
    }))
    .expect("构造请求");

    let actual = handle(&runtime, &req).expect("处理成功");
    let expected: Value = serde_json::from_str(EXPECTED).expect("期望 JSON 合法");

    assert_close(&actual, &expected, "$");
}

/// 递归比对：对象键集合相等、数组顺序严格、数字容差 1e-6。
fn assert_close(actual: &Value, expected: &Value, path: &str) {
    match (actual, expected) {
        (Value::Object(left), Value::Object(right)) => {
            let mut left_keys: Vec<&String> = left.keys().collect();
            let mut right_keys: Vec<&String> = right.keys().collect();
            left_keys.sort();
            right_keys.sort();
            assert_eq!(left_keys, right_keys, "{path} 键集合不一致");
            for (key, value) in right {
                assert_close(&left[key], value, &format!("{path}.{key}"));
            }
        }
        (Value::Array(left), Value::Array(right)) => {
            assert_eq!(left.len(), right.len(), "{path} 数组长度不一致");
            for (index, (a, b)) in left.iter().zip(right).enumerate() {
                assert_close(a, b, &format!("{path}[{index}]"));
            }
        }
        (Value::Number(left), Value::Number(right)) => {
            let a = left.as_f64().expect("数字");
            let b = right.as_f64().expect("数字");
            let tolerance = 1e-6 * a.abs().max(b.abs()).max(1.0);
            assert!((a - b).abs() <= tolerance, "{path} 数值不等: {a} vs {b}");
        }
        _ => assert_eq!(actual, expected, "{path} 值不一致"),
    }
}

#[test]
fn comparator_rejects_different_lengths() {
    let left = serde_json::json!({"years": [2024]});
    let right = serde_json::json!({"years": [2024, 2023]});
    let result = std::panic::catch_unwind(|| assert_close(&left, &right, "$"));
    assert!(result.is_err(), "长度不同必须断言失败");
}

#[test]
fn yearly_analysis_matches_python_golden() {
    compare(
        "/yearly_analysis",
        &[("year", "2024")],
        include_str!("../../../fixtures/expected/yearly_analysis_2024.json"),
    );
}

#[test]
fn monthly_analysis_matches_python_golden() {
    compare(
        "/monthly_analysis",
        &[("year", "2024"), ("month", "12")],
        include_str!("../../../fixtures/expected/monthly_analysis_2024_12.json"),
    );
}

#[test]
fn category_analysis_matches_python_golden() {
    compare(
        "/category_analysis",
        &[("year", "2024")],
        include_str!("../../../fixtures/expected/category_analysis_2024.json"),
    );
}

#[test]
fn transactions_match_python_golden() {
    compare(
        "/transactions",
        &[("type", "支出"), ("per_page", "50")],
        include_str!("../../../fixtures/expected/transactions_expense.json"),
    );
}

#[test]
fn analysis_matches_python_golden() {
    compare(
        "/analysis",
        &[("year", "2024")],
        include_str!("../../../fixtures/expected/analysis_2024.json"),
    );
}

#[test]
fn yearly_data_matches_python_golden() {
    compare(
        "/yearly_data",
        &[("year", "2024")],
        include_str!("../../../fixtures/expected/yearly_data_2024.json"),
    );
}

#[test]
fn monthly_data_matches_python_golden() {
    compare(
        "/monthly_data",
        &[("year", "2024"), ("month", "12")],
        include_str!("../../../fixtures/expected/monthly_data_2024_12.json"),
    );
}

#[test]
fn filtered_monthly_analysis_matches_python_golden() {
    compare(
        "/filtered_monthly_analysis",
        &[],
        include_str!("../../../fixtures/expected/filtered_monthly_analysis_all.json"),
    );
}

// ===== 其余 18 个接口 =====

const CSV: &str = "交易时间,交易分类,交易对方,商品说明,收/支,金额,收/付款方式,交易状态\n\
2024-01-15 12:30:00,餐饮美食,星巴克,咖啡,支出,28.50,余额宝,交易成功\n";

fn demo_runtime() -> Runtime {
    let dir = tempfile::tempdir().expect("临时目录");
    let paths = DataPaths::new(dir.path().to_path_buf());
    paths.ensure_layout().expect("创建布局");
    std::mem::forget(dir);
    Runtime::with_dataset(paths, demo_dataset().expect("演示数据"))
}

fn empty_runtime() -> Runtime {
    let dir = tempfile::tempdir().expect("临时目录");
    let paths = DataPaths::new(dir.path().to_path_buf());
    paths.ensure_layout().expect("创建布局");
    std::mem::forget(dir);
    Runtime::with_dataset(paths, bill_core::Dataset::default())
}

fn compare_runtime(
    runtime: &Runtime,
    method: &str,
    path: &str,
    query: &[(&str, &str)],
    body: serde_json::Value,
    expected: &str,
) {
    let query: std::collections::BTreeMap<String, String> = query
        .iter()
        .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
        .collect();
    let request: ApiRequest = serde_json::from_value(serde_json::json!({
        "method": method,
        "path": path,
        "query": query,
        "body": body,
    }))
    .expect("构造请求");
    let actual = handle(runtime, &request).expect("处理成功");
    let expected: Value = serde_json::from_str(expected).expect("期望 JSON 合法");
    assert_close(&actual, &expected, path);
}

fn compare_demo(path: &str, query: &[(&str, &str)], expected: &str) {
    compare_runtime(&demo_runtime(), "GET", path, query, Value::Null, expected);
}

#[test]
fn metadata_endpoints_match_python_golden() {
    compare_demo(
        "/available_dates",
        &[],
        include_str!("../../../fixtures/expected/available_dates.json"),
    );
    compare_demo(
        "/categories",
        &[],
        include_str!("../../../fixtures/expected/categories.json"),
    );
    compare_demo(
        "/category_available_dates",
        &[],
        include_str!("../../../fixtures/expected/category_available_dates.json"),
    );
    compare_demo(
        "/category_expenses",
        &[],
        include_str!("../../../fixtures/expected/category_expenses.json"),
    );
}

#[test]
fn category_detail_and_trend_match_python_golden() {
    compare_demo(
        "/category_detail/2024-12/%E9%A4%90%E9%A5%AE%E7%BE%8E%E9%A3%9F",
        &[],
        include_str!("../../../fixtures/expected/category_detail_2024_12_food.json"),
    );
    compare_demo(
        "/category_trend/%E9%A4%90%E9%A5%AE%E7%BE%8E%E9%A3%9F",
        &[],
        include_str!("../../../fixtures/expected/category_trend_food.json"),
    );
}

#[test]
fn misc_read_endpoints_match_python_golden() {
    compare_demo(
        "/top_transactions",
        &[("limit", "5"), ("min_amount", "1000")],
        include_str!("../../../fixtures/expected/top_transactions.json"),
    );
    compare_demo(
        "/daily_data",
        &[("year", "2024")],
        include_str!("../../../fixtures/expected/daily_data_2024.json"),
    );
    compare_demo(
        "/time_analysis",
        &[("year", "2024")],
        include_str!("../../../fixtures/expected/time_analysis_2024.json"),
    );
    compare_demo(
        "/overview_data",
        &[("year", "2024")],
        include_str!("../../../fixtures/expected/overview_data_2024.json"),
    );
}

#[test]
fn summary_matches_python_golden_with_frozen_month() {
    let runtime = demo_runtime().with_current_month("2026-11");
    compare_runtime(
        &runtime,
        "GET",
        "/summary",
        &[],
        Value::Null,
        include_str!("../../../fixtures/expected/summary.json"),
    );
}

#[test]
fn files_and_session_endpoints_match_python_golden() {
    compare_runtime(
        &empty_runtime(),
        "GET",
        "/files",
        &[],
        Value::Null,
        include_str!("../../../fixtures/expected/files_empty.json"),
    );
    compare_runtime(
        &empty_runtime(),
        "GET",
        "/session/status",
        &[],
        Value::Null,
        include_str!("../../../fixtures/expected/session_status_fresh.json"),
    );
}

#[test]
fn demo_toggle_matches_python_golden() {
    let runtime = empty_runtime();
    compare_runtime(
        &runtime,
        "POST",
        "/demo/enter",
        &[],
        Value::Null,
        include_str!("../../../fixtures/expected/demo_enter.json"),
    );
    compare_runtime(
        &runtime,
        "POST",
        "/demo/exit",
        &[],
        Value::Null,
        include_str!("../../../fixtures/expected/demo_exit.json"),
    );
}

fn upload_source() -> (tempfile::TempDir, String) {
    let dir = tempfile::tempdir().expect("来源目录");
    let path = dir.path().join("alipay_record.csv");
    std::fs::write(&path, CSV).expect("写来源");
    let path = path.to_string_lossy().into_owned();
    (dir, path)
}

#[test]
fn upload_matches_python_golden() {
    let runtime = empty_runtime();
    let (_dir, source) = upload_source();
    compare_runtime(
        &runtime,
        "POST",
        "/upload",
        &[],
        serde_json::json!({"paths": [source], "provider": "alipay"}),
        include_str!("../../../fixtures/expected/upload_single.json"),
    );
}

#[test]
fn delete_matches_python_golden() {
    let runtime = empty_runtime();
    let (_dir, source) = upload_source();
    let upload: ApiRequest = serde_json::from_value(serde_json::json!({
        "method": "POST",
        "path": "/upload",
        "body": {"paths": [source], "provider": "alipay"},
    }))
    .expect("上传请求");
    handle(&runtime, &upload).expect("上传");

    compare_runtime(
        &runtime,
        "DELETE",
        "/files/alipay_record.csv",
        &[],
        Value::Null,
        include_str!("../../../fixtures/expected/delete_file.json"),
    );
}

#[test]
fn clear_matches_python_golden() {
    let runtime = empty_runtime();
    let (_dir, source) = upload_source();
    let upload: ApiRequest = serde_json::from_value(serde_json::json!({
        "method": "POST",
        "path": "/upload",
        "body": {"paths": [source], "provider": "alipay"},
    }))
    .expect("上传请求");
    handle(&runtime, &upload).expect("上传");

    compare_runtime(
        &runtime,
        "POST",
        "/clear_data",
        &[],
        Value::Null,
        include_str!("../../../fixtures/expected/clear_data.json"),
    );
}
