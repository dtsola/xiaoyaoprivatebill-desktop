//! P4-5 / P4-8 / P4-9 契约测试：import_bill、审计与演示禁写。

use std::sync::Arc;

use bill_api::Runtime;
use bill_core::{DataPaths, Dataset, Session};
use bill_mcp::write::{import_bill, ImportParams, DEMO_WRITE_ERROR};
use serde_json::json;

const ALIPAY: &str = "交易时间,交易分类,交易对方,商品说明,收/支,金额,收/付款方式,交易状态\n\
2024-01-15 12:30:00,餐饮美食,星巴克,咖啡,支出,28.50,余额宝,交易成功\n";

struct Env {
    runtime: Arc<Runtime>,
    source_dir: tempfile::TempDir,
}

fn setup() -> Env {
    let data = tempfile::tempdir().expect("数据目录");
    let paths = DataPaths::new(data.path().to_path_buf());
    paths.ensure_layout().expect("创建布局");
    std::mem::forget(data);

    let source_dir = tempfile::tempdir().expect("来源目录");
    std::fs::write(source_dir.path().join("alipay.csv"), ALIPAY).expect("写来源");

    Env {
        runtime: Arc::new(Runtime::with_dataset(paths, Dataset::default())),
        source_dir,
    }
}

fn params(paths: &[String]) -> ImportParams {
    ImportParams {
        paths: paths.to_vec(),
    }
}

fn audit_lines(env: &Env) -> Vec<serde_json::Value> {
    std::fs::read_to_string(env.runtime.paths.audit_file())
        .unwrap_or_default()
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect()
}

#[test]
fn import_succeeds_and_writes_intent_and_result_audit() {
    let env = setup();
    let source = env.source_dir.path().join("alipay.csv");
    let response = import_bill(
        &env.runtime,
        &params(&[source.to_string_lossy().into_owned()]),
    );

    let results = response["results"].as_array().expect("结果");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["ok"], json!(true));
    assert_eq!(results[0]["filename"], json!("alipay.csv"));

    // 数据集已刷新。
    let dataset = env.runtime.dataset();
    assert_eq!(dataset.available_years(), vec![2024]);
    assert!(env.runtime.paths.bills_dir().join("alipay.csv").exists());

    // 审计两行：intent + result，且不含 Token。
    let lines = audit_lines(&env);
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0]["phase"], json!("intent"));
    assert_eq!(lines[1]["phase"], json!("result"));
    assert_eq!(lines[1]["actor"]["tool"], json!("import_bill"));
    assert_eq!(lines[1]["result"], json!("ok"));
    let raw = std::fs::read_to_string(env.runtime.paths.audit_file()).expect("审计");
    assert!(!raw.contains("Bearer"));
}

#[test]
fn import_reports_per_file_errors_without_blocking_others() {
    let env = setup();
    let good = env.source_dir.path().join("alipay.csv");
    let missing = env.source_dir.path().join("missing.csv");
    let response = import_bill(
        &env.runtime,
        &params(&[
            good.to_string_lossy().into_owned(),
            missing.to_string_lossy().into_owned(),
        ]),
    );

    let results = response["results"].as_array().expect("结果");
    assert_eq!(results[0]["ok"], json!(true));
    assert_eq!(results[1]["ok"], json!(false));
    assert!(
        results[1]["error"].as_str().expect("错误").contains("读取"),
        "错误信息: {}",
        results[1]["error"]
    );

    let lines = audit_lines(&env);
    assert_eq!(lines[1]["result"], json!("err"));
    assert_eq!(lines[1]["target"]["imported"], json!(1));
    assert_eq!(lines[1]["target"]["failed"], json!(1));
}

#[test]
fn demo_mode_rejects_writes_without_audit() {
    let env = setup();
    Session::enter_demo(&env.runtime.paths).expect("进入演示");
    let source = env.source_dir.path().join("alipay.csv");

    let response = import_bill(
        &env.runtime,
        &params(&[source.to_string_lossy().into_owned()]),
    );

    assert_eq!(response["success"], json!(false));
    assert_eq!(response["error"], json!(DEMO_WRITE_ERROR));
    assert!(
        !env.runtime.paths.bills_dir().join("alipay.csv").exists(),
        "演示模式不得落盘"
    );
    assert!(audit_lines(&env).is_empty(), "被拒绝的写入不产生审计");
}

#[test]
fn import_rejects_empty_and_oversized_batches() {
    let env = setup();

    let empty = import_bill(&env.runtime, &params(&[]));
    assert_eq!(empty["success"], json!(false));

    let many: Vec<String> = (0..21).map(|index| format!("C:/x/{index}.csv")).collect();
    let oversized = import_bill(&env.runtime, &params(&many));
    assert_eq!(oversized["success"], json!(false));
    assert!(
        oversized["error"].as_str().expect("错误").contains("1-20"),
        "错误信息: {}",
        oversized["error"]
    );
}
