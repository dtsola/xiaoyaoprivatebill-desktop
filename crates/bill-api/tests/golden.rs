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
