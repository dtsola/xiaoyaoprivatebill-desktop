//! 10 个只读工具的实现。
//!
//! 全部复用 `bill_api::handle`，保证与界面返回一致。

use std::collections::{BTreeMap, BTreeSet};

use bill_api::{handle, ApiRequest, Runtime};
use bill_core::{DataPaths, Session};
use rmcp::schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::audit;

/// 年度类工具参数。
#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct YearlyParams {
    /// 年份；缺省为全部数据。
    pub year: Option<i32>,
    /// 金额下限（含）。
    pub min_amount: Option<f64>,
    /// 金额上限（不含）。
    pub max_amount: Option<f64>,
}

/// 月度类工具参数。
#[derive(Debug, Deserialize, JsonSchema)]
pub struct MonthlyParams {
    /// 年份。
    pub year: i32,
    /// 月份 1–12。
    pub month: u32,
    /// 金额下限（含）。
    pub min_amount: Option<f64>,
    /// 金额上限（不含）。
    pub max_amount: Option<f64>,
}

/// 分类分析参数。
#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct CategoryParams {
    /// 分类；缺省为列表模式。
    pub category: Option<String>,
    /// 年份。
    pub year: Option<i32>,
}

/// 交易搜索参数。
#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct SearchParams {
    /// 页码，从 1 开始。
    pub page: Option<usize>,
    /// 每页条数。
    pub per_page: Option<usize>,
    /// 年份。
    pub year: Option<i32>,
    /// 月份。
    pub month: Option<u32>,
    /// 日期 `YYYY-MM-DD`。
    pub date: Option<String>,
    /// 小时。
    pub hour: Option<u32>,
    /// 分类。
    pub category: Option<String>,
    /// 金额下限（含）。
    pub min_amount: Option<f64>,
    /// 金额上限（含）。
    pub max_amount: Option<f64>,
    /// 收/支。
    pub r#type: Option<String>,
    /// 正则搜索。
    pub search: Option<String>,
}

/// 时间分析参数。
#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct TimeParams {
    /// 年份。
    pub year: Option<i32>,
    /// 金额筛选：all / large / small。
    pub filter: Option<String>,
}

/// 审计日志参数。
#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct AuditParams {
    /// 返回条数，默认 50，上限 500。
    pub limit: Option<usize>,
    /// 按工具名过滤。
    pub tool: Option<String>,
    /// 起始时间（RFC 3339）。
    pub since: Option<String>,
}

/// 列出账单文件。
pub fn list_bills(runtime: &Runtime) -> Value {
    let files = bill_services::files::list_files(&runtime.paths).unwrap_or_default();
    let items: Vec<Value> = files
        .iter()
        .map(|file| {
            json!({
                "name": file.name,
                "size": file.size,
                "size_formatted": format_size(file.size),
                "source": file.source,
                "imported_at": file.imported_at,
            })
        })
        .collect();
    json!({
        "files": items,
        "demo": Session::load(&runtime.paths).is_demo,
    })
}

/// 数据概览。
pub fn get_overview(runtime: &Runtime) -> Value {
    let dataset = runtime.dataset();
    let months: Vec<String> = dataset
        .transactions
        .iter()
        .map(|row| row.month.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .rev()
        .collect();

    let bill_count = bill_services::files::list_files(&runtime.paths)
        .map(|files| files.len())
        .unwrap_or_default();

    json!({
        "years": dataset.available_years(),
        "months": months,
        "transaction_count": dataset.transactions.len(),
        "bill_count": bill_count,
        "is_demo": Session::load(&runtime.paths).is_demo,
    })
}

/// 汇总数据。
pub fn get_summary(runtime: &Runtime) -> Value {
    call(runtime, "/summary", BTreeMap::new())
}

/// 年度分析（返回 `data`）。
pub fn get_yearly_analysis(runtime: &Runtime, params: &YearlyParams) -> Value {
    let mut query = BTreeMap::new();
    if let Some(year) = params.year {
        query.insert("year".to_owned(), year.to_string());
    }
    insert_amounts(&mut query, params.min_amount, params.max_amount);
    call(runtime, "/yearly_analysis", query)["data"].clone()
}

/// 月度分析（返回 `data`）。
pub fn get_monthly_analysis(runtime: &Runtime, params: &MonthlyParams) -> Value {
    let mut query = BTreeMap::new();
    query.insert("year".to_owned(), params.year.to_string());
    query.insert("month".to_owned(), params.month.to_string());
    insert_amounts(&mut query, params.min_amount, params.max_amount);
    call(runtime, "/monthly_analysis", query)["data"].clone()
}

/// 分类分析。
pub fn get_category_analysis(runtime: &Runtime, params: &CategoryParams) -> Value {
    let mut query = BTreeMap::new();
    if let Some(category) = &params.category {
        query.insert("category".to_owned(), category.clone());
    }
    if let Some(year) = params.year {
        query.insert("year".to_owned(), year.to_string());
    }
    call(runtime, "/category_analysis", query)
}

/// 交易搜索。
pub fn search_transactions(runtime: &Runtime, params: &SearchParams) -> Value {
    let mut query = BTreeMap::new();
    let mut push = |key: &str, value: Option<String>| {
        if let Some(value) = value {
            query.insert(key.to_owned(), value);
        }
    };
    push("page", params.page.map(|value| value.to_string()));
    push("per_page", params.per_page.map(|value| value.to_string()));
    push("year", params.year.map(|value| value.to_string()));
    push("month", params.month.map(|value| value.to_string()));
    push("date", params.date.clone());
    push("hour", params.hour.map(|value| value.to_string()));
    push("category", params.category.clone());
    push(
        "min_amount",
        params.min_amount.map(|value| value.to_string()),
    );
    push(
        "max_amount",
        params.max_amount.map(|value| value.to_string()),
    );
    push("type", params.r#type.clone());
    push("search", params.search.clone());
    call(runtime, "/transactions", query)
}

/// 时间分析。
pub fn get_time_analysis(runtime: &Runtime, params: &TimeParams) -> Value {
    let mut query = BTreeMap::new();
    if let Some(year) = params.year {
        query.insert("year".to_owned(), year.to_string());
    }
    if let Some(filter) = &params.filter {
        query.insert("filter".to_owned(), filter.clone());
    }
    call(runtime, "/time_analysis", query)
}

/// 消费洞察（返回 `data`）。
pub fn get_insights(runtime: &Runtime, params: &YearlyParams) -> Value {
    let mut query = BTreeMap::new();
    if let Some(year) = params.year {
        query.insert("year".to_owned(), year.to_string());
    }
    insert_amounts(&mut query, params.min_amount, params.max_amount);
    call(runtime, "/analysis", query)["data"].clone()
}

/// 审计日志。
pub fn get_audit_log(paths: &DataPaths, params: &AuditParams) -> Value {
    audit::read(
        paths,
        params.limit.unwrap_or(audit::DEFAULT_LIMIT),
        params.tool.as_deref(),
        params.since.as_deref(),
    )
}

fn call(runtime: &Runtime, path: &str, query: BTreeMap<String, String>) -> Value {
    let request = ApiRequest {
        method: "GET".to_owned(),
        path: path.to_owned(),
        query,
        body: Value::Null,
    };
    handle(runtime, &request).unwrap_or_else(|error| json!({ "success": false, "error": error }))
}

fn insert_amounts(query: &mut BTreeMap<String, String>, min: Option<f64>, max: Option<f64>) {
    if let Some(min) = min {
        query.insert("min_amount".to_owned(), min.to_string());
    }
    if let Some(max) = max {
        query.insert("max_amount".to_owned(), max.to_string());
    }
}

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
