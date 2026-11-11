//! P4-5 / P4-8 / P4-9 契约测试：import_bill、审计与演示禁写。

use std::sync::Arc;

use bill_api::Runtime;
use bill_core::{DataPaths, Dataset, Session};
use bill_mcp::write::{
    clear_data, delete_bill, import_bill, restore_bill, ClearParams, DeleteParams, ImportParams,
    RestoreParams, DEMO_WRITE_ERROR,
};
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

fn write_bill(env: &Env, name: &str) {
    std::fs::write(env.runtime.paths.bills_dir().join(name), ALIPAY).expect("写账单");
}

#[test]
fn delete_moves_to_trash_and_returns_id() {
    let env = setup();
    write_bill(&env, "test.csv");

    let response = delete_bill(
        &env.runtime,
        &DeleteParams {
            filename: "test.csv".into(),
        },
    );
    assert_eq!(response["ok"], json!(true));
    assert_eq!(response["filename"], json!("test.csv"));
    let trash_id = response["trash_id"].as_str().expect("trash_id");
    assert!(!trash_id.is_empty());
    assert!(!env.runtime.paths.bills_dir().join("test.csv").exists());

    let lines = audit_lines(&env);
    assert_eq!(lines[0]["actor"]["tool"], json!("delete_bill"));
    assert_eq!(lines[0]["phase"], json!("intent"));
    assert_eq!(lines[1]["phase"], json!("result"));
}

#[test]
fn delete_missing_file_reports_error() {
    let env = setup();
    let response = delete_bill(
        &env.runtime,
        &DeleteParams {
            filename: "nope.csv".into(),
        },
    );
    assert_eq!(response["ok"], json!(false));
    assert_eq!(response["error"], json!("文件不存在"));
}

#[test]
fn restore_brings_file_back_and_is_single_use() {
    let env = setup();
    write_bill(&env, "test.csv");
    let deleted = delete_bill(
        &env.runtime,
        &DeleteParams {
            filename: "test.csv".into(),
        },
    );
    let trash_id = deleted["trash_id"].as_str().expect("trash_id");

    let restored = restore_bill(
        &env.runtime,
        &RestoreParams {
            trash_id: trash_id.into(),
        },
    );
    assert_eq!(restored["ok"], json!(true));
    assert_eq!(restored["restored"], json!(["test.csv"]));
    assert!(env.runtime.paths.bills_dir().join("test.csv").exists());

    let again = restore_bill(
        &env.runtime,
        &RestoreParams {
            trash_id: trash_id.into(),
        },
    );
    assert_eq!(again["ok"], json!(false));
    assert!(
        again["error"].as_str().expect("错误").contains("已恢复"),
        "错误信息: {}",
        again["error"]
    );
}

#[test]
fn restore_adds_suffix_when_name_taken() {
    let env = setup();
    write_bill(&env, "test.csv");
    let deleted = delete_bill(
        &env.runtime,
        &DeleteParams {
            filename: "test.csv".into(),
        },
    );
    let trash_id = deleted["trash_id"].as_str().expect("trash_id");

    // 恢复前又导入同名文件，恢复时应自动加序号。
    write_bill(&env, "test.csv");
    let restored = restore_bill(
        &env.runtime,
        &RestoreParams {
            trash_id: trash_id.into(),
        },
    );

    assert_eq!(restored["restored"], json!(["test_1.csv"]));
    assert!(env.runtime.paths.bills_dir().join("test_1.csv").exists());
}

#[test]
fn demo_mode_rejects_delete_and_restore() {
    let env = setup();
    Session::enter_demo(&env.runtime.paths).expect("进入演示");

    let deleted = delete_bill(
        &env.runtime,
        &DeleteParams {
            filename: "test.csv".into(),
        },
    );
    assert_eq!(deleted["error"], json!(DEMO_WRITE_ERROR));

    let restored = restore_bill(
        &env.runtime,
        &RestoreParams {
            trash_id: "20240101000000-0000".into(),
        },
    );
    assert_eq!(restored["error"], json!(DEMO_WRITE_ERROR));
}

#[test]
fn clear_requires_confirm() {
    let env = setup();
    let response = clear_data(&env.runtime, &ClearParams { confirm: false });
    assert_eq!(response["success"], json!(false));
    assert!(
        response["error"]
            .as_str()
            .expect("错误")
            .contains("confirm"),
        "错误信息: {}",
        response["error"]
    );
}

#[test]
fn clear_snapshots_before_removing() {
    let env = setup();
    write_bill(&env, "a.csv");
    write_bill(&env, "b.csv");

    let response = clear_data(&env.runtime, &ClearParams { confirm: true });
    assert_eq!(response["ok"], json!(true));
    assert_eq!(response["removed_count"], json!(2));
    let snapshot_id = response["snapshot_id"].as_str().expect("snapshot_id");
    assert!(!snapshot_id.is_empty());

    assert!(!env.runtime.paths.bills_dir().join("a.csv").exists());
    assert!(!env.runtime.paths.bills_dir().join("b.csv").exists());
    let snapshot_dir = env.runtime.paths.snapshots_dir().join(snapshot_id);
    assert!(snapshot_dir.join("files").join("a.csv").exists());
    assert!(snapshot_dir.join("files").join("b.csv").exists());

    let lines = audit_lines(&env);
    assert_eq!(lines[0]["actor"]["tool"], json!("clear_data"));
    assert_eq!(lines[1]["target"]["snapshot_id"], json!(snapshot_id));
}

#[test]
fn clear_on_empty_dataset_still_snapshots() {
    let env = setup();
    let response = clear_data(&env.runtime, &ClearParams { confirm: true });

    assert_eq!(response["ok"], json!(true));
    assert_eq!(response["removed_count"], json!(0));
    assert!(env
        .runtime
        .paths
        .snapshots_dir()
        .join(response["snapshot_id"].as_str().expect("id"))
        .join("manifest.json")
        .exists());
}

#[test]
fn demo_mode_rejects_clear() {
    let env = setup();
    write_bill(&env, "a.csv");
    Session::enter_demo(&env.runtime.paths).expect("进入演示");

    let response = clear_data(&env.runtime, &ClearParams { confirm: true });
    assert_eq!(response["error"], json!(DEMO_WRITE_ERROR));
    assert!(env.runtime.paths.bills_dir().join("a.csv").exists());
}
