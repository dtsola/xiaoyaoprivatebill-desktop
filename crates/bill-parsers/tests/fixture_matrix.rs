//! P1-10 fixture 矩阵：编码、BOM、空行、退款、空值。
//!
//! 夹具由 `tools/fixtures/gen_parser_fixtures.py` 生成。

use bill_parsers::alipay::parse_alipay_csv;
use bill_parsers::wechat::parse_wechat_csv;

const ALIPAY_GBK: &[u8] = include_bytes!("../../../fixtures/constructed/alipay_gbk.csv");
const ALIPAY_BOM: &[u8] = include_bytes!("../../../fixtures/constructed/alipay_utf8_bom.csv");
const WECHAT_BOM: &[u8] = include_bytes!("../../../fixtures/constructed/wechat_utf8_bom.csv");
const WECHAT_BLANK: &[u8] = include_bytes!("../../../fixtures/constructed/wechat_blank_lines.csv");

#[test]
fn parses_gbk_file_with_blank_lines_and_junk_footer() {
    let rows = parse_alipay_csv(ALIPAY_GBK).expect("GBK 夹具应解析成功");
    assert_eq!(rows.len(), 4, "空行与尾部说明不应产生交易");
    assert_eq!(rows[0].counterparty, "星巴克");
    assert_eq!(rows[3].description, "地铁票");
}

#[test]
fn alipay_refund_status_does_not_flip_amount_due_to_d1() {
    let rows = parse_alipay_csv(ALIPAY_GBK).expect("解析成功");
    let refund_row = &rows[1];
    assert_eq!(refund_row.status, "退款成功");
    assert!(!refund_row.is_refund, "D1：退款判定恒为 false");
    assert!((refund_row.amount - 100.0).abs() < 1e-9, "金额保持原值");
}

#[test]
fn empty_cells_become_empty_strings_not_nan() {
    let rows = parse_alipay_csv(ALIPAY_GBK).expect("解析成功");
    let row = &rows[2];
    assert_eq!(row.counterparty, "");
    assert_eq!(row.description, "");
}

#[test]
fn parses_utf8_bom_file() {
    let rows = parse_alipay_csv(ALIPAY_BOM).expect("BOM 夹具应解析成功");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].description, "拿铁");
}

#[test]
fn parses_wechat_bom_with_refund() {
    let rows = parse_wechat_csv(WECHAT_BOM).expect("微信 BOM 夹具应解析成功");
    assert_eq!(rows.len(), 2);
    assert!(!rows[0].is_refund);
    assert!(rows[1].is_refund);
    assert!((rows[1].amount + 5.0).abs() < 1e-9);
}

#[test]
fn parses_wechat_file_with_blank_lines() {
    let rows = parse_wechat_csv(WECHAT_BLANK).expect("空行夹具应解析成功");
    assert_eq!(rows.len(), 2);
}
