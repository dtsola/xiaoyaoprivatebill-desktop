//! MCP 服务层：传输、鉴权、生命周期与 14 个工具。
//!
//! 只读工具复用 `bill_api::handle`，保证返回与界面一致；
//! 写工具见后续单元。

pub mod audit;
pub mod auth;
pub mod lifecycle;
pub mod token;
pub mod tools;
pub mod write;

use std::sync::Arc;

use axum::Router;
use bill_api::Runtime;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock};
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::{StreamableHttpServerConfig, StreamableHttpService};
use rmcp::{tool, tool_handler, tool_router, ErrorData, ServerHandler};

pub use auth::Auth;

/// MCP 监听端口。只允许绑定回环地址。
pub const DEFAULT_PORT: u16 = 17650;

/// MCP 路由前缀。
pub const ROUTE_PATH: &str = "/mcp";

/// 请求体上限 1MB（见技术方案 03 §9.1）。
pub const MAX_REQUEST_BODY_BYTES: usize = 1024 * 1024;

/// MCP 服务处理器。
#[derive(Clone)]
pub struct BillMcp {
    runtime: Arc<Runtime>,
}

impl BillMcp {
    /// 以共享运行时构造。
    pub fn new(runtime: Arc<Runtime>) -> Self {
        Self { runtime }
    }

    fn json(value: serde_json::Value) -> Result<CallToolResult, ErrorData> {
        Ok(CallToolResult::success(vec![ContentBlock::text(
            value.to_string(),
        )]))
    }
}

#[tool_router]
impl BillMcp {
    #[tool(
        description = "列出已导入的账单文件（名称、大小、来源、导入时间）",
        annotations(title = "列出账单文件", read_only_hint = true)
    )]
    async fn list_bills(&self) -> Result<CallToolResult, ErrorData> {
        Self::json(tools::list_bills(&self.runtime))
    }

    #[tool(
        description = "数据概览：可用年份、月份、交易笔数、文件数、是否演示模式",
        annotations(title = "数据概览", read_only_hint = true)
    )]
    async fn get_overview(&self) -> Result<CallToolResult, ErrorData> {
        Self::json(tools::get_overview(&self.runtime))
    }

    #[tool(
        description = "汇总数据：总支出、总收入、结余、月均支出、当前月支出",
        annotations(title = "汇总数据", read_only_hint = true)
    )]
    async fn get_summary(&self) -> Result<CallToolResult, ErrorData> {
        Self::json(tools::get_summary(&self.runtime))
    }

    #[tool(
        description = "年度分析：按月支出/收入、分类占比、同比变化",
        annotations(title = "年度分析", read_only_hint = true)
    )]
    async fn get_yearly_analysis(
        &self,
        Parameters(params): Parameters<tools::YearlyParams>,
    ) -> Result<CallToolResult, ErrorData> {
        Self::json(tools::get_yearly_analysis(&self.runtime, &params))
    }

    #[tool(
        description = "月度分析：当月统计、每日曲线、分类构成、环比",
        annotations(title = "月度分析", read_only_hint = true)
    )]
    async fn get_monthly_analysis(
        &self,
        Parameters(params): Parameters<tools::MonthlyParams>,
    ) -> Result<CallToolResult, ErrorData> {
        Self::json(tools::get_monthly_analysis(&self.runtime, &params))
    }

    #[tool(
        description = "分类分析：分类列表，或指定分类的明细与趋势",
        annotations(title = "分类分析", read_only_hint = true)
    )]
    async fn get_category_analysis(
        &self,
        Parameters(params): Parameters<tools::CategoryParams>,
    ) -> Result<CallToolResult, ErrorData> {
        Self::json(tools::get_category_analysis(&self.runtime, &params))
    }

    #[tool(
        description = "搜索交易记录：支持分页、日期、分类、金额、收支与正则搜索",
        annotations(title = "搜索交易", read_only_hint = true)
    )]
    async fn search_transactions(
        &self,
        Parameters(params): Parameters<tools::SearchParams>,
    ) -> Result<CallToolResult, ErrorData> {
        Self::json(tools::search_transactions(&self.runtime, &params))
    }

    #[tool(
        description = "时间分析：24 小时分布与工作日/周末对比",
        annotations(title = "时间分析", read_only_hint = true)
    )]
    async fn get_time_analysis(
        &self,
        Parameters(params): Parameters<tools::TimeParams>,
    ) -> Result<CallToolResult, ErrorData> {
        Self::json(tools::get_time_analysis(&self.runtime, &params))
    }

    #[tool(
        description = "消费洞察：23 项分析（商户、场景、订阅、图表数据、故事等）",
        annotations(title = "消费洞察", read_only_hint = true)
    )]
    async fn get_insights(
        &self,
        Parameters(params): Parameters<tools::YearlyParams>,
    ) -> Result<CallToolResult, ErrorData> {
        Self::json(tools::get_insights(&self.runtime, &params))
    }

    #[tool(
        description = "读取审计日志：按工具与起始时间过滤，默认最近 50 条",
        annotations(title = "审计日志", read_only_hint = true)
    )]
    async fn get_audit_log(
        &self,
        Parameters(params): Parameters<tools::AuditParams>,
    ) -> Result<CallToolResult, ErrorData> {
        Self::json(tools::get_audit_log(&self.runtime.paths, &params))
    }

    #[tool(
        description = "导入支付宝/微信账单文件（绝对路径，1-20 个），逐个返回结果",
        annotations(title = "导入账单", destructive_hint = false)
    )]
    async fn import_bill(
        &self,
        Parameters(params): Parameters<write::ImportParams>,
    ) -> Result<CallToolResult, ErrorData> {
        Self::json(write::import_bill(&self.runtime, &params))
    }

    #[tool(
        description = "删除账单文件（移入回收站，可恢复）",
        annotations(title = "删除账单", destructive_hint = true)
    )]
    async fn delete_bill(
        &self,
        Parameters(params): Parameters<write::DeleteParams>,
    ) -> Result<CallToolResult, ErrorData> {
        Self::json(write::delete_bill(&self.runtime, &params))
    }

    #[tool(
        description = "从回收站恢复账单（传入删除时返回的 trash_id）",
        annotations(title = "恢复账单", destructive_hint = false)
    )]
    async fn restore_bill(
        &self,
        Parameters(params): Parameters<write::RestoreParams>,
    ) -> Result<CallToolResult, ErrorData> {
        Self::json(write::restore_bill(&self.runtime, &params))
    }

    #[tool(
        description = "清空全部账单（必须先快照，需 confirm=true）",
        annotations(title = "清空账单", destructive_hint = true)
    )]
    async fn clear_data(
        &self,
        Parameters(params): Parameters<write::ClearParams>,
    ) -> Result<CallToolResult, ErrorData> {
        Self::json(write::clear_data(&self.runtime, &params))
    }
}

/// 去掉 schema 根部的 `$schema` 声明。
///
/// rmcp 基于 schemars 生成 draft 2020-12 的 `$schema` 字段，部分 MCP 客户端
/// （如小遥Claw 内置的 AJV）未注册该 dialect，会报
/// `no schema with key or ref "https://json-schema.org/draft/2020-12/schema"`。
/// 本项目的工具参数都是扁平结构，去掉声明后按客户端默认 draft 校验即可。
fn strip_schema_meta(schema: &rmcp::model::JsonObject) -> rmcp::model::JsonObject {
    let mut map = schema.clone();
    map.remove("$schema");
    map
}

fn sanitize_tool(mut tool: rmcp::model::Tool) -> rmcp::model::Tool {
    tool.input_schema = Arc::new(strip_schema_meta(&tool.input_schema));
    if let Some(output) = &tool.output_schema {
        tool.output_schema = Some(Arc::new(strip_schema_meta(output)));
    }
    tool
}

#[tool_handler]
impl ServerHandler for BillMcp {
    // 覆盖宏生成的 list_tools：把所有工具的 `$schema` 声明去掉，
    // 兼容未注册 draft 2020-12 元 schema 的 MCP 客户端。
    async fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<rmcp::model::ListToolsResult, ErrorData> {
        let supports_cache_hints = context
            .protocol_version()
            .is_some_and(|version| version >= rmcp::model::ProtocolVersion::V_2026_07_28);
        Ok(rmcp::model::ListToolsResult {
            result_type: Some(rmcp::model::ResultType::COMPLETE),
            tools: Self::tool_router()
                .list_all()
                .into_iter()
                .map(sanitize_tool)
                .collect(),
            meta: None,
            next_cursor: None,
            ttl_ms: supports_cache_hints.then_some(0),
            cache_scope: supports_cache_hints.then_some(rmcp::model::CacheScope::Public),
        })
    }

    fn get_tool(&self, name: &str) -> Option<rmcp::model::Tool> {
        Self::tool_router().get(name).cloned().map(sanitize_tool)
    }
}

/// 构建 Streamable HTTP 服务。
pub fn mcp_service(runtime: Arc<Runtime>) -> StreamableHttpService<BillMcp, LocalSessionManager> {
    let config = StreamableHttpServerConfig::default()
        .with_allowed_hosts(["127.0.0.1", "localhost"])
        .with_max_request_body_bytes(MAX_REQUEST_BODY_BYTES);

    StreamableHttpService::new(
        move || Ok(BillMcp::new(runtime.clone())),
        Arc::new(LocalSessionManager::default()),
        config,
    )
}

/// 构建仅包含 MCP 路由的 axum Router（**不含鉴权**）。
///
/// 仅用于传输层测试；生产入口是 [`router_with_token`]。
pub fn router(runtime: Arc<Runtime>) -> Router {
    Router::new().nest_service(ROUTE_PATH, mcp_service(runtime))
}

/// 构建带 Bearer + Origin 校验的 MCP Router。
pub fn router_with_token(runtime: Arc<Runtime>, token: &str) -> Router {
    Router::new()
        .nest_service(ROUTE_PATH, mcp_service(runtime))
        .layer(axum::middleware::from_fn_with_state(
            Auth::new(token),
            auth::bearer_check,
        ))
        .layer(axum::extract::DefaultBodyLimit::max(MAX_REQUEST_BODY_BYTES))
}
