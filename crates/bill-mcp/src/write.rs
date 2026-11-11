//! 写工具：导入账单（后续单元补充删除、恢复与清空）。

use std::path::PathBuf;

use bill_api::Runtime;
use bill_core::Session;
use rmcp::schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::audit;

/// 单次导入允许的最大文件数。
pub const MAX_IMPORT_PATHS: usize = 20;

/// 演示模式下的统一错误。
pub const DEMO_WRITE_ERROR: &str = "演示模式下禁止写入";

/// 导入参数。
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ImportParams {
    /// 待导入文件的绝对路径，1–20 个。
    pub paths: Vec<String>,
}

/// 批量导入，逐个文件返回结果；单个失败不影响其他文件。
pub fn import_bill(runtime: &Runtime, params: &ImportParams) -> Value {
    if Session::load(&runtime.paths).is_demo {
        return json!({ "success": false, "error": DEMO_WRITE_ERROR });
    }
    if params.paths.is_empty() || params.paths.len() > MAX_IMPORT_PATHS {
        return json!({ "success": false, "error": "paths 数量必须在 1-20 之间" });
    }

    let _ = audit::record(
        &runtime.paths,
        "import_bill",
        "intent",
        "ok",
        json!({ "files": params.paths.iter().filter_map(|path| file_name(path)).collect::<Vec<_>>() }),
        None,
    );

    let mut results = Vec::new();
    for path in &params.paths {
        let source = PathBuf::from(path);
        match bill_services::files::import_paths(&runtime.paths, &[source]) {
            Ok(names) => results.push(json!({
                "path": path,
                "ok": true,
                "filename": names.first().cloned().unwrap_or_default(),
            })),
            Err(error) => results.push(json!({
                "path": path,
                "ok": false,
                "error": error.to_string(),
            })),
        }
    }

    runtime.refresh();

    let failed = results
        .iter()
        .filter(|result| result["ok"] == json!(false))
        .count();
    let target = json!({
        "files": params.paths.iter().filter_map(|path| file_name(path)).collect::<Vec<_>>(),
        "imported": results.len() - failed,
        "failed": failed,
    });
    let _ = audit::record(
        &runtime.paths,
        "import_bill",
        "result",
        if failed == 0 { "ok" } else { "err" },
        target,
        None,
    );

    json!({ "results": results, "warnings": [] })
}

fn file_name(raw: &str) -> Option<String> {
    PathBuf::from(raw)
        .file_name()
        .map(|value| value.to_string_lossy().into_owned())
}
