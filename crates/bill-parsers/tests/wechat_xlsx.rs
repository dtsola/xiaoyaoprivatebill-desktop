//! P1-5 契约测试：微信 XLSX 解析。
//!
//! 夹具由 `openpyxl` 生成，时间单元格为 Excel 日期序列。

use bill_parsers::wechat::parse_wechat_xlsx;
use chrono::NaiveDate;

const BILL: &[u8] = include_bytes!("../../../fixtures/constructed/wechat_bill.xlsx");
const NO_AMOUNT: &[u8] = include_bytes!("../../../fixtures/constructed/wechat_bill_no_amount.xlsx");
const INVALID: &[u8] = include_bytes!("../../../fixtures/constructed/wechat_bill_invalid.xlsx");

#[test]
fn parses_xlsx_and_converts_excel_dates() {
    let rows = parse_wechat_xlsx(BILL).expect("解析应成功");
    assert_eq!(rows.len(), 3);

    let first = &rows[0];
    assert_eq!(
        first.time,
        NaiveDate::from_ymd_opt(2024, 1, 15)
            .expect("日期合法")
            .and_hms_opt(12, 30, 0)
            .expect("时间合法"),
        "Excel 日期序列应转换为本地时间"
    );
    assert_eq!(first.month, "2024-01");
    assert_eq!(first.date, "2024-01-15");
    assert_eq!(first.category, "商户消费");
    assert_eq!(first.description, "咖啡");
    assert_eq!(first.counterparty, "星巴克");
    assert_eq!(first.payment_method, "零钱");
    assert_eq!(first.status, "支付成功");
    assert!((first.amount - 28.50).abs() < 1e-9, "金额(元) 应映射为金额");
    assert!(!first.is_refund);
    assert_eq!(first.source, "微信");
}

#[test]
fn refund_row_becomes_negative() {
    let rows = parse_wechat_xlsx(BILL).expect("解析应成功");

    assert!(rows[2].is_refund, "已退款应标记为退款");
    assert!(
        (rows[2].amount + 100.00).abs() < 1e-9,
        "退款金额应为负绝对值"
    );
}

#[test]
fn errors_when_amount_column_missing() {
    let error = parse_wechat_xlsx(NO_AMOUNT).expect_err("缺少金额列必须报错");
    assert!(
        error.to_string().contains("金额"),
        "错误信息应指出缺失字段: {error}"
    );
}

#[test]
fn errors_when_header_missing() {
    let error = parse_wechat_xlsx(INVALID).expect_err("缺少表头必须报错");
    let message = error.to_string();
    assert!(
        message.contains("无法找到微信账单表头行") || message.contains("不是有效的微信账单文件"),
        "错误信息应可读: {message}"
    );
}

#[test]
fn errors_on_non_xlsx_bytes() {
    let error = parse_wechat_xlsx(b"not an xlsx").expect_err("非 XLSX 必须报错");
    assert!(
        error.to_string().contains("Excel"),
        "错误信息应说明 Excel 读取失败: {error}"
    );
}
