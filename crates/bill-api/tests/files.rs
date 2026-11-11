//! P1-8 契约测试：文件管理接口的外层形状。

use std::fs;
use std::path::PathBuf;

use bill_api::{handle, ApiRequest, Runtime};
use bill_core::DataPaths;
use serde_json::json;

const ALIPAY: &str = "交易时间,交易分类,交易对方,商品说明,收/支,金额,收/付款方式,交易状态\n\
2024-01-15 12:30:00,餐饮美食,星巴克,咖啡,支出,28.50,余额宝,交易成功\n";

struct Env {
    _data: tempfile::TempDir,
    _source: tempfile::TempDir,
    runtime: Runtime,
    source_file: PathBuf,
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

fn request(method: &str, path: &str, body: serde_json::Value) -> ApiRequest {
    serde_json::from_value(json!({
        "method": method,
        "path": path,
        "body": body,
    }))
    .expect("构造请求")
}

#[test]
fn upload_returns_filename_and_message() {
    let env = setup();
    let req = request(
        "POST",
        "/upload",
        json!({ "paths": [env.source_file.to_string_lossy()] }),
    );

    let response = handle(&env.runtime, &req).expect("处理成功");
    assert_eq!(response["success"], true);
    assert_eq!(response["filename"], "alipay.csv");
    assert_eq!(response["message"], "文件上传成功");
}

#[test]
fn upload_without_paths_returns_error_envelope() {
    let env = setup();
    let response = handle(&env.runtime, &request("POST", "/upload", json!({}))).expect("处理成功");

    assert_eq!(response["success"], false);
    assert_eq!(response["error"], "没有文件被上传");
}

#[test]
fn files_lists_uploaded_files_with_size_formatted() {
    let env = setup();
    let _ = handle(
        &env.runtime,
        &request(
            "POST",
            "/upload",
            json!({ "paths": [env.source_file.to_string_lossy()] }),
        ),
    )
    .expect("上传");

    let response = handle(&env.runtime, &request("GET", "/files", json!(null))).expect("处理成功");
    let files = response["files"].as_array().expect("files 数组");
    assert_eq!(files.len(), 1);
    assert_eq!(files[0]["name"], "alipay.csv");
    assert_eq!(files[0]["source"], "支付宝");
    assert_eq!(files[0]["size"], ALIPAY.len());
    assert!(files[0]["size_formatted"].as_str().is_some());
}

#[test]
fn delete_returns_success_and_second_time_not_found() {
    let env = setup();
    let _ = handle(
        &env.runtime,
        &request(
            "POST",
            "/upload",
            json!({ "paths": [env.source_file.to_string_lossy()] }),
        ),
    )
    .expect("上传");

    let first = handle(
        &env.runtime,
        &request("DELETE", "/files/alipay.csv", json!(null)),
    )
    .expect("处理成功");
    assert_eq!(first["success"], true);

    let second = handle(
        &env.runtime,
        &request("DELETE", "/files/alipay.csv", json!(null)),
    )
    .expect("处理成功");
    assert_eq!(second["success"], false);
    assert_eq!(second["error"], "文件不存在");
}

#[test]
fn delete_rejects_invalid_extension() {
    let env = setup();
    let response = handle(
        &env.runtime,
        &request("DELETE", "/files/settings.json", json!(null)),
    )
    .expect("处理成功");

    assert_eq!(response["success"], false);
    assert_eq!(response["error"], "无效的文件名");
}

#[test]
fn clear_returns_message() {
    let env = setup();
    let _ = handle(
        &env.runtime,
        &request(
            "POST",
            "/upload",
            json!({ "paths": [env.source_file.to_string_lossy()] }),
        ),
    )
    .expect("上传");

    let response =
        handle(&env.runtime, &request("POST", "/clear_data", json!(null))).expect("处理成功");
    assert_eq!(response["success"], true);
    assert_eq!(response["message"], "数据已清除");
}
