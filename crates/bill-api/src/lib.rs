//! API 分发层：把前端 IPC 请求映射到服务函数。
//!
//! P1 只覆盖文件管理与 `/available_years`；P2 补齐其余接口。

use std::path::PathBuf;
use std::sync::RwLock;

use bill_core::{DataPaths, Dataset, Session};
use bill_services::analysis::{
    daily_data, filtered_monthly_analysis, monthly_analysis, monthly_data, overview_data, summary,
    time_analysis, top_transactions, yearly_analysis, yearly_data, MonthlyQuery, YearlyQuery,
};
use bill_services::category::{
    category_analysis, category_available_dates, category_detail, category_expenses, category_trend,
};
use bill_services::charts::{
    analyze_sankey, generate_boxplot_data, generate_chord_data, generate_funnel_data,
    generate_heatmap_data, generate_pareto_data, generate_quadrant_data, generate_radar_data,
    generate_themeriver_data, generate_wordcloud_data,
};
use bill_services::dataset::{demo_dataset, load_dataset};
use bill_services::files::{clear_data, delete_file, import_paths, list_files};
use bill_services::insights::{
    analyze_brand_loyalty, analyze_engel_coefficient, analyze_habits, analyze_inflation,
    analyze_latte_factor, analyze_merchants, analyze_nighttime_spending, analyze_payment_methods,
    analyze_scenarios, analyze_subscriptions, analyze_weekend_vs_monday, generate_smart_tags,
    generate_story_data,
};
use bill_services::transactions::{list_transactions, TransactionQuery};
use chrono::Datelike;
use serde::Deserialize;
use serde_json::{json, Value};

/// 前端 IPC 请求。
#[derive(Debug, Deserialize)]
pub struct ApiRequest {
    pub method: String,
    /// 不含 `/api` 前缀，例如 `/available_years`。
    pub path: String,
    #[serde(default)]
    pub query: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    pub body: Value,
}

/// 运行时状态：数据目录 + 内存数据集。
#[derive(Debug)]
pub struct Runtime {
    /// 数据目录布局。
    pub paths: DataPaths,
    dataset: RwLock<Dataset>,
}

impl Runtime {
    /// 以数据目录构造运行时。
    ///
    /// `session.json` 声明演示模式时加载内嵌演示数据，否则扫描 `bills/`。
    pub fn new(paths: DataPaths) -> Self {
        let dataset = load_initial(&paths);
        Self {
            paths,
            dataset: RwLock::new(dataset),
        }
    }

    /// 直接指定数据集（测试与后续导入重建使用）。
    pub fn with_dataset(paths: DataPaths, dataset: Dataset) -> Self {
        Self {
            paths,
            dataset: RwLock::new(dataset),
        }
    }

    /// 当前数据集快照。
    pub fn dataset(&self) -> Dataset {
        self.dataset
            .read()
            .map(|guard| guard.clone())
            .unwrap_or_default()
    }

    /// 写操作后重建数据集。
    pub fn refresh(&self) {
        if let Ok(mut guard) = self.dataset.write() {
            *guard = load_initial(&self.paths);
        }
    }
}

fn load_initial(paths: &DataPaths) -> Dataset {
    if Session::load(paths).is_demo {
        return demo_dataset().unwrap_or_default();
    }
    load_dataset(&paths.bills_dir()).unwrap_or_default()
}

/// 处理一个 API 请求。
///
/// 业务失败以 `{ success: false, error }` 返回；传输级错误才返回 `Err`。
pub fn handle(runtime: &Runtime, req: &ApiRequest) -> Result<Value, String> {
    match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/available_years") => Ok(json!({
            "success": true,
            "years": runtime.dataset().available_years()
        })),
        ("GET", "/yearly_analysis") => Ok(handle_yearly(runtime, &req.query)),
        ("GET", "/monthly_analysis") => Ok(handle_monthly(runtime, &req.query)),
        ("GET", "/yearly_data") => Ok({
            let year = req
                .query
                .get("year")
                .and_then(|value| value.parse::<i32>().ok());
            let filter = req.query.get("filter").map(String::as_str).unwrap_or("all");
            yearly_data(&runtime.dataset(), year, filter)
        }),
        ("GET", "/monthly_data") => Ok({
            let year = req
                .query
                .get("year")
                .and_then(|value| value.parse::<i32>().ok());
            let month = req
                .query
                .get("month")
                .and_then(|value| value.parse::<u32>().ok());
            let filter = req.query.get("filter").map(String::as_str).unwrap_or("all");
            monthly_data(&runtime.dataset(), year, month, filter)
        }),
        ("GET", "/filtered_monthly_analysis") => Ok({
            let filter = req.query.get("filter").map(String::as_str).unwrap_or("all");
            filtered_monthly_analysis(&runtime.dataset(), filter)
        }),
        ("GET", "/overview_data") => Ok({
            let year = req
                .query
                .get("year")
                .and_then(|value| value.parse::<i32>().ok());
            let filter = req.query.get("filter").map(String::as_str).unwrap_or("all");
            overview_data(&runtime.dataset(), year, filter)
        }),
        ("GET", "/summary") => Ok(summary(
            &runtime.dataset(),
            &chrono::Local::now().format("%Y-%m").to_string(),
        )),
        ("GET", "/top_transactions") => Ok({
            let limit = req
                .query
                .get("limit")
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(10);
            let min_amount = req
                .query
                .get("min_amount")
                .and_then(|value| value.parse::<f64>().ok())
                .unwrap_or(1000.0);
            top_transactions(&runtime.dataset(), limit, min_amount)
        }),
        ("GET", "/daily_data") => Ok({
            let year = req
                .query
                .get("year")
                .and_then(|value| value.parse::<i32>().ok());
            let filter = req.query.get("filter").map(String::as_str).unwrap_or("all");
            daily_data(&runtime.dataset(), year, filter)
        }),
        ("GET", "/time_analysis") => Ok({
            let year = req
                .query
                .get("year")
                .and_then(|value| value.parse::<i32>().ok());
            let filter = req.query.get("filter").map(String::as_str).unwrap_or("all");
            time_analysis(&runtime.dataset(), year, filter)
        }),
        ("GET", "/analysis") => Ok(handle_analysis(runtime, &req.query)),
        ("GET", "/transactions") => Ok(handle_transactions(runtime, &req.query)),
        ("GET", "/available_dates") => Ok(handle_available_dates(runtime)),
        ("GET", "/categories") => Ok(handle_categories(runtime)),
        ("GET", "/category_expenses") => Ok(category_expenses(&runtime.dataset())),
        ("GET", "/category_analysis") => Ok({
            let year = req
                .query
                .get("year")
                .and_then(|value| value.parse::<i32>().ok());
            category_analysis(
                &runtime.dataset(),
                req.query.get("category").map(String::as_str),
                year,
            )
        }),
        ("GET", "/category_available_dates") => Ok(category_available_dates(&runtime.dataset())),
        ("GET", "/files") => Ok(handle_list(runtime)),
        ("GET", "/session/status") => Ok(handle_session_status(runtime)),
        ("POST", "/demo/enter") => Ok(handle_demo(runtime, true)),
        ("POST", "/demo/exit") => Ok(handle_demo(runtime, false)),
        ("POST", "/upload") => Ok(handle_upload(runtime, &req.body)),
        ("POST", "/clear_data") => Ok(handle_clear(runtime)),
        _ => {
            if let Some(rest) = req.path.strip_prefix("/category_detail/") {
                let mut parts = rest.splitn(2, '/');
                let month = percent_decode(parts.next().unwrap_or_default());
                let category = percent_decode(parts.next().unwrap_or_default());
                return Ok(category_detail(&runtime.dataset(), &month, &category));
            }
            if let Some(category) = req.path.strip_prefix("/category_trend/") {
                return Ok(category_trend(
                    &runtime.dataset(),
                    &percent_decode(category),
                ));
            }
            if let Some(name) = req.path.strip_prefix("/files/") {
                if req.method == "DELETE" {
                    return Ok(handle_delete(runtime, name));
                }
            }
            Ok(json!({
                "success": false,
                "error": format!("接口不存在: {}", req.path)
            }))
        }
    }
}

fn handle_yearly(runtime: &Runtime, query: &std::collections::BTreeMap<String, String>) -> Value {
    let year = query
        .get("year")
        .and_then(|value| value.parse::<i32>().ok());
    let min_amount = query
        .get("min_amount")
        .and_then(|value| value.parse::<f64>().ok());
    let max_amount = query
        .get("max_amount")
        .and_then(|value| value.parse::<f64>().ok());

    let data = yearly_analysis(
        &runtime.dataset(),
        &YearlyQuery {
            year,
            min_amount,
            max_amount,
        },
    );
    json!({ "success": true, "data": data })
}

fn handle_analysis(runtime: &Runtime, query: &std::collections::BTreeMap<String, String>) -> Value {
    // 与 Web 版一致：`if year:` / `if min_amount:` 视 0 为未设置。
    let year = query
        .get("year")
        .and_then(|value| value.parse::<i32>().ok())
        .filter(|value| *value != 0);
    let min_amount = query
        .get("min_amount")
        .and_then(|value| value.parse::<f64>().ok())
        .filter(|value| *value != 0.0);
    let max_amount = query
        .get("max_amount")
        .and_then(|value| value.parse::<f64>().ok())
        .filter(|value| *value != 0.0);

    let dataset = runtime.dataset();
    let filtered = Dataset {
        transactions: dataset
            .transactions
            .iter()
            .filter(|row| {
                year.is_none_or(|value| row.time.year() == value)
                    && min_amount.is_none_or(|value| row.amount >= value)
                    && max_amount.is_none_or(|value| row.amount < value)
            })
            .cloned()
            .collect(),
        files: dataset.files.clone(),
    };

    json!({
        "success": true,
        "data": {
            "merchant_analysis": analyze_merchants(&filtered),
            "scenario_analysis": analyze_scenarios(&filtered),
            "habit_analysis": analyze_habits(&filtered),
            "latte_factor": analyze_latte_factor(&filtered),
            "nighttime_analysis": analyze_nighttime_spending(&filtered),
            "subscription_analysis": analyze_subscriptions(&filtered),
            "inflation_analysis": analyze_inflation(&filtered),
            "brand_loyalty": analyze_brand_loyalty(&filtered),
            "sankey_data": analyze_sankey(&filtered),
            "engel_coefficient": analyze_engel_coefficient(&filtered),
            "weekend_monday": analyze_weekend_vs_monday(&filtered),
            "story_data": generate_story_data(&filtered),
            "tags": generate_smart_tags(&filtered),
            "payment_analysis": analyze_payment_methods(&filtered),
            "chord_data": generate_chord_data(&filtered),
            "funnel_data": generate_funnel_data(&filtered),
            "quadrant_data": generate_quadrant_data(&filtered),
            "radar_data": generate_radar_data(&filtered),
            "wordcloud_data": generate_wordcloud_data(&filtered),
            "themeriver_data": generate_themeriver_data(&filtered),
            "boxplot_data": generate_boxplot_data(&filtered),
            "heatmap_data": generate_heatmap_data(&filtered),
            "pareto_data": generate_pareto_data(&filtered),
        }
    })
}

fn handle_monthly(runtime: &Runtime, query: &std::collections::BTreeMap<String, String>) -> Value {
    let number = |key: &str| query.get(key).and_then(|value| value.parse::<f64>().ok());
    let (Some(year), Some(month)) = (number("year"), number("month")) else {
        return json!({
            "success": false,
            "error": "获取月度分析失败: 缺少年份或月份"
        });
    };

    let data = monthly_analysis(
        &runtime.dataset(),
        &MonthlyQuery {
            year: year as i32,
            month: month as u32,
            min_amount: number("min_amount"),
            max_amount: number("max_amount"),
        },
    );
    json!({ "success": true, "data": data })
}

fn handle_transactions(
    runtime: &Runtime,
    query: &std::collections::BTreeMap<String, String>,
) -> Value {
    let number = |key: &str| query.get(key).and_then(|value| value.parse::<f64>().ok());
    let query = TransactionQuery {
        page: number("page").map(|value| value as usize).unwrap_or(1),
        per_page: number("per_page").map(|value| value as usize).unwrap_or(0),
        year: number("year").map(|value| value as i32),
        month: number("month").map(|value| value as u32),
        date: query.get("date").cloned(),
        hour: number("hour").map(|value| value as u32),
        category: query.get("category").cloned(),
        min_amount: number("min_amount"),
        max_amount: number("max_amount"),
        r#type: query.get("type").cloned(),
        search: query.get("search").cloned(),
    };
    list_transactions(&runtime.dataset(), &query)
}

fn handle_available_dates(runtime: &Runtime) -> Value {
    let dataset = runtime.dataset();
    let mut months: Vec<String> = dataset
        .transactions
        .iter()
        .map(|row| row.month.clone())
        .collect();
    months.sort_by(|left, right| right.cmp(left));
    months.dedup();

    json!({
        "success": true,
        "months": months,
        "years": dataset.available_years(),
    })
}

fn handle_categories(runtime: &Runtime) -> Value {
    let dataset = runtime.dataset();
    let mut categories: Vec<String> = dataset
        .transactions
        .iter()
        .map(|row| row.category.clone())
        .collect();
    categories.sort();
    categories.dedup();

    json!({ "success": true, "categories": categories })
}

fn handle_list(runtime: &Runtime) -> Value {
    match list_files(&runtime.paths) {
        Ok(files) => {
            let items: Vec<Value> = files
                .iter()
                .map(|file| {
                    json!({
                        "name": file.name,
                        "size": file.size,
                        "size_formatted": format_size(file.size),
                        "source": source_code(&file.source),
                    })
                })
                .collect();
            json!({ "files": items })
        }
        Err(error) => json!({ "success": false, "error": error.to_string() }),
    }
}

fn handle_upload(runtime: &Runtime, body: &Value) -> Value {
    let Some(paths) = body.get("paths").and_then(|value| value.as_array()) else {
        return json!({ "success": false, "error": "没有文件被上传" });
    };
    let sources: Vec<PathBuf> = paths
        .iter()
        .filter_map(|value| value.as_str())
        .map(PathBuf::from)
        .collect();

    match import_paths(&runtime.paths, &sources) {
        Ok(names) => {
            runtime.refresh();
            json!({
            "success": true,
            "filename": names.first().cloned().unwrap_or_default(),
                "filenames": names,
                "message": "文件上传成功"
            })
        }
        Err(error) => json!({ "success": false, "error": error.to_string() }),
    }
}

fn handle_delete(runtime: &Runtime, raw_name: &str) -> Value {
    let name = percent_decode(raw_name);
    match delete_file(&runtime.paths, &name) {
        Ok(_) => {
            runtime.refresh();
            json!({ "success": true })
        }
        Err(error) => json!({ "success": false, "error": error.to_string() }),
    }
}

fn handle_session_status(runtime: &Runtime) -> Value {
    let session = Session::load(&runtime.paths);
    if !session.is_active() {
        return json!({ "active": false, "is_demo": false });
    }
    json!({
        "active": true,
        "is_demo": session.is_demo,
        "user_id": session.user_id.unwrap_or_default(),
        "session_start": session.session_start.unwrap_or_default(),
    })
}

fn handle_demo(runtime: &Runtime, enter: bool) -> Value {
    let result = if enter {
        Session::enter_demo(&runtime.paths)
    } else {
        Session::exit_demo(&runtime.paths)
    };
    match result {
        Ok(_) => {
            runtime.refresh();
            json!({ "success": true })
        }
        Err(error) => json!({ "success": false, "error": error.to_string() }),
    }
}

fn handle_clear(runtime: &Runtime) -> Value {
    match clear_data(&runtime.paths) {
        Ok(_) => {
            runtime.refresh();
            json!({ "success": true, "message": "数据已清除" })
        }
        Err(error) => json!({ "success": false, "error": error.to_string() }),
    }
}

/// 与 Web 版 `detect_file_source` 的返回码保持一致。
fn source_code(source: &str) -> &str {
    match source {
        "微信" => "wechat",
        "支付宝" => "alipay",
        _ => "unknown",
    }
}

/// 复刻 Flask `format_file_size`：B/KB/MB/GB/TB，保留一位小数。
fn format_size(size: u64) -> String {
    let mut value = size as f64;
    for unit in ["B", "KB", "MB", "GB"] {
        if value < 1024.0 {
            return format!("{value:.1} {unit}");
        }
        value /= 1024.0;
    }
    format!("{value:.1} TB")
}

/// 最小百分号解码，用于 `DELETE /files/<name>`。
fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[index + 1..index + 3]).unwrap_or("");
            if let Ok(byte) = u8::from_str_radix(hex, 16) {
                out.push(byte);
                index += 3;
                continue;
            }
        }
        out.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn runtime() -> (tempfile::TempDir, Runtime) {
        let dir = tempfile::tempdir().expect("临时目录");
        let paths = DataPaths::new(dir.path().to_path_buf());
        paths.ensure_layout().expect("创建布局");
        (dir, Runtime::new(paths))
    }

    #[test]
    fn available_years_is_empty_without_data() {
        let (_dir, runtime) = runtime();
        let req: ApiRequest = serde_json::from_value(json!({
            "method": "GET",
            "path": "/available_years"
        }))
        .expect("构造请求");

        let result = handle(&runtime, &req).expect("处理成功");
        assert_eq!(result, json!({"success": true, "years": []}));
    }

    #[test]
    fn unknown_route_returns_error_envelope() {
        let (_dir, runtime) = runtime();
        let req: ApiRequest = serde_json::from_value(json!({
            "method": "GET",
            "path": "/nope"
        }))
        .expect("构造请求");

        let result = handle(&runtime, &req).expect("处理成功");
        assert_eq!(result["success"], false);
        assert_eq!(result["error"], "接口不存在: /nope");
    }

    #[test]
    fn query_and_body_default_when_missing() {
        let req: ApiRequest = serde_json::from_value(json!({
            "method": "GET",
            "path": "/available_years"
        }))
        .expect("构造请求");

        assert!(req.query.is_empty());
        assert_eq!(req.body, Value::Null);
    }

    #[test]
    fn format_size_matches_python() {
        assert_eq!(format_size(0), "0.0 B");
        assert_eq!(format_size(1023), "1023.0 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
    }

    #[test]
    fn percent_decode_handles_chinese_and_plus() {
        assert_eq!(percent_decode("a%20b.csv"), "a b.csv");
        assert_eq!(percent_decode("plain.csv"), "plain.csv");
    }
}
