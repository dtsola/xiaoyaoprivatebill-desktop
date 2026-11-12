# 架构决策 - MCP 工具 schema 去除 draft 2020-12 声明

**决策日期**：2026年11月12日

## 背景

小遥Claw 通过 MCP 调用带参数的工具（`get_yearly_analysis`、`get_monthly_analysis`、`get_category_analysis`、`get_insights` 等）时统一报错：

```
no schema with key or ref "https://json-schema.org/draft/2020-12/schema"
```

无参数工具（`get_overview`、`get_summary`）正常。原因是 rmcp 基于 schemars 生成的参数 schema 根部带 `$schema: draft/2020-12`，而客户端的 AJV 实例未注册该 draft 的元 schema；只有带参数的工具有非空 schema，才会触发校验。

## 解决方案

在 MCP ServerHandler 中覆盖 `list_tools` 与 `get_tool`，对每个工具去掉 `inputSchema` / `outputSchema` 根部的 `$schema` 字段，其余结构保持不变（工具参数均为扁平类型，不依赖 `$defs` 或跨文档 `$ref`）。

- 实现位置：`crates/bill-mcp/src/lib.rs` 的 `sanitize_tool` / `strip_schema_meta`
- 回归测试：`crates/bill-mcp/tests/mcp_e2e.rs` 断言 `tools/list` 中所有 schema 均不含 `$schema`
- 客户端侧无需改动；标准 MCP 客户端对缺少 `$schema` 的 schema 按默认 draft 处理

## 影响

- 小遥Claw 等未注册 draft 2020-12 的客户端可以正常调用全部 14 个工具。
- 工具 schema 不再声明 dialect；未来若引入复杂 schema（`oneOf`、`$defs`、递归引用），需要重新评估，必要时改为显式提供 draft-07 schema 或推动客户端注册新 draft。
- 参数校验仍由服务端完成，去掉声明不降低安全性。

---

**决策编号**：AD-20261112-001
