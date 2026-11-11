//! P2-9 收尾：演示模式与会话状态路由。

use std::fs;

use bill_api::{handle, ApiRequest, Runtime};
use bill_core::DataPaths;
use serde_json::{json, Value};

const ALIPAY: &str = "交易时间,交易分类,交易对方,商品说明,收/支,金额,收/付款方式,交易状态\n\
2024-01-15 12:30:00,餐饮美食,星巴克,咖啡,支出,28.50,余额宝,交易成功\n";

struct Env {
    _data: tempfile::TempDir,
    _source: tempfile::TempDir,
    runtime: Runtime,
    source_file: std::path::PathBuf,
}

fn setup() -> Env {
    let data = tempfile::tempdir().expect("数据目录");
    let source = tempfile::tempdir().expect("来源目录");
    let paths = DataPaths::new(data.path().to_path_buf());
    paths.ensure_layout().expect("创建布局");

    let source_file = source.path().join("alipay.csv");
    fs::write(&source_file, ALIPAY).expect("写来源文件");

    Env {
        _data: data,
        _source: source,
        runtime: Runtime::new(paths),
        source_file,
    }
}

fn request(method: &str, path: &str, body: Value) -> ApiRequest {
    serde_json::from_value(json!({
        "method": method,
        "path": path,
        "body": body,
    }))
    .expect("构造请求")
}

fn status(runtime: &Runtime) -> Value {
    handle(runtime, &request("GET", "/session/status", Value::Null)).expect("状态")
}

#[test]
fn fresh_session_is_inactive() {
    let env = setup();
    assert_eq!(
        status(&env.runtime),
        json!({"active": false, "is_demo": false})
    );
}

#[test]
fn demo_mode_switches_dataset_and_status() {
    let env = setup();

    let response =
        handle(&env.runtime, &request("POST", "/demo/enter", Value::Null)).expect("进入演示");
    assert_eq!(response, json!({"success": true}));

    assert_eq!(
        status(&env.runtime),
        json!({
            "active": true,
            "is_demo": true,
            "user_id": "demo_user",
            "session_start": "",
        })
    );

    // 演示数据已加载（内嵌 7,396 条）。
    let years = handle(
        &env.runtime,
        &request("GET", "/available_years", Value::Null),
    )
    .expect("年份");
    assert!(
        !years["years"].as_array().expect("年份数组").is_empty(),
        "演示模式应能读到数据"
    );

    let response =
        handle(&env.runtime, &request("POST", "/demo/exit", Value::Null)).expect("退出演示");
    assert_eq!(response, json!({"success": true}));
    assert_eq!(
        status(&env.runtime),
        json!({"active": false, "is_demo": false})
    );

    let years = handle(
        &env.runtime,
        &request("GET", "/available_years", Value::Null),
    )
    .expect("年份");
    assert_eq!(years["years"], json!([]), "退出后回到空的 bills/");
}

#[test]
fn successful_import_activates_local_session() {
    let env = setup();
    let response = handle(
        &env.runtime,
        &request(
            "POST",
            "/upload",
            json!({"paths": [env.source_file.to_string_lossy()], "provider": "alipay"}),
        ),
    )
    .expect("上传");
    assert_eq!(response["success"], true);

    let status = status(&env.runtime);
    assert_eq!(status["active"], true);
    assert_eq!(status["is_demo"], false);
    assert_eq!(status["user_id"], "user_local");
    assert!(
        !status["session_start"]
            .as_str()
            .unwrap_or_default()
            .is_empty(),
        "首次导入写入 session_start"
    );
}

#[test]
fn clear_data_keeps_user_identity() {
    let env = setup();
    handle(
        &env.runtime,
        &request(
            "POST",
            "/upload",
            json!({"paths": [env.source_file.to_string_lossy()], "provider": "alipay"}),
        ),
    )
    .expect("上传");

    let response =
        handle(&env.runtime, &request("POST", "/clear_data", Value::Null)).expect("清空");
    assert_eq!(response["success"], true);

    let status = status(&env.runtime);
    assert_eq!(status["active"], true, "clear_data 不清除 user_id");
    assert_eq!(status["user_id"], "user_local");
}
