//! 支付宝 CSV 解析，行为基准为 Web 版 `parsers/alipay.py`。
//!
//! 故意保留两个缺陷：
//! - D1：退款判定使用「状态行第一列列名」，实际恒为 false；
//! - D2：不做 `金额（元）→金额` 映射，缺少 `金额` 列直接报错。

use bill_core::Transaction;

use crate::common::{decode_gbk_or_utf8, first_field, line_offset, parse_time, text_field};
use crate::error::ParseError;

/// 解析支付宝 CSV 字节流。
///
/// 编码优先 GBK，解码出错时回退 UTF-8。无法解析时间的行会被跳过。
pub fn parse_alipay_csv(bytes: &[u8]) -> Result<Vec<Transaction>, ParseError> {
    let text = decode_gbk_or_utf8(bytes)?;

    let header_offset = line_offset(&text, "交易时间").ok_or(ParseError::AlipayHeader)?;
    let status_offset = line_offset(&text, "交易状态");

    // D1：状态列取状态行的第一列列名，而不是「交易状态」列。
    let status_column = status_offset
        .and_then(|offset| first_field(&text[offset..]))
        .unwrap_or_default();

    let body = &text.as_bytes()[header_offset..];
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(body);

    let headers = reader
        .headers()
        .map_err(|error| ParseError::MissingColumn(format!("表头读取失败: {error}")))?
        .clone();

    let time_index = headers
        .iter()
        .position(|name| name == "交易时间")
        .ok_or(ParseError::AlipayHeader)?;
    let amount_index = headers
        .iter()
        .position(|name| name == "金额")
        .ok_or_else(|| ParseError::MissingColumn("金额".into()))?;
    let status_index = headers.iter().position(|name| name == status_column);

    let mut rows = Vec::new();
    for record in reader.records() {
        let record = match record {
            Ok(record) => record,
            Err(_) => continue,
        };

        let Some(time) = parse_time(record.get(time_index).unwrap_or_default()) else {
            continue;
        };
        let Some(amount) = record
            .get(amount_index)
            .and_then(|raw| raw.trim().parse::<f64>().ok())
        else {
            continue;
        };

        let is_refund = status_index
            .and_then(|index| record.get(index))
            .is_some_and(|value| value == "退款成功" || value == "交易关闭");
        let amount = if is_refund { -amount } else { amount };

        rows.push(Transaction {
            time,
            month: time.format("%Y-%m").to_string(),
            date: time.format("%Y-%m-%d").to_string(),
            direction: text_field(&record, &headers, "收/支").unwrap_or_else(|| "/".into()),
            amount,
            category: text_field(&record, &headers, "交易分类").unwrap_or_else(|| "其他".into()),
            description: text_field(&record, &headers, "商品说明")
                .or_else(|| text_field(&record, &headers, "商品"))
                .unwrap_or_default(),
            counterparty: text_field(&record, &headers, "交易对方")
                .unwrap_or_else(|| "未知".into()),
            payment_method: text_field(&record, &headers, "收/付款方式").unwrap_or_default(),
            status: text_field(&record, &headers, "交易状态").unwrap_or_default(),
            is_refund,
            source: "支付宝".into(),
        });
    }

    Ok(rows)
}
