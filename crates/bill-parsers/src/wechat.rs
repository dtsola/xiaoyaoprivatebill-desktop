//! 微信账单解析，行为基准为 Web 版 `parsers/wechat.py`。

use bill_core::Transaction;

use crate::common::{decode_utf8_sig, header_index, line_offset_matching, parse_time};
use crate::error::ParseError;

/// 解析微信 CSV 字节流。
///
/// 表头必须同时包含「交易时间」与「交易类型」；无法解析时间或金额的行会被跳过。
pub fn parse_wechat_csv(bytes: &[u8]) -> Result<Vec<Transaction>, ParseError> {
    let text = decode_utf8_sig(bytes)?;
    let header_offset = line_offset_matching(&text, |line| {
        line.contains("交易时间") && line.contains("交易类型")
    })
    .ok_or(ParseError::WechatHeader)?;

    let body = &text.as_bytes()[header_offset..];
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(body);

    let headers = reader
        .headers()
        .map_err(|error| ParseError::MissingColumn(format!("表头读取失败: {error}")))?
        .clone();

    let time_index = header_index(&headers, &["交易时间"]).ok_or(ParseError::WechatHeader)?;
    let amount_index = header_index(&headers, &["金额(元)", "金额"])
        .ok_or_else(|| ParseError::MissingColumn("金额(元)".into()))?;
    let category_index = header_index(&headers, &["交易类型", "交易分类"]);
    let description_index = header_index(&headers, &["商品", "商品说明"]);
    let status_index = header_index(&headers, &["当前状态", "交易状态"]);
    let payment_index = header_index(&headers, &["支付方式", "收/付款方式"]);
    let direction_index = header_index(&headers, &["收/支"]);
    let counterparty_index = header_index(&headers, &["交易对方"]);

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
            .and_then(|raw| clean_amount(raw).parse::<f64>().ok())
        else {
            continue;
        };

        let status = value_at(&record, status_index).unwrap_or_default();
        let is_refund = is_refund_status(&status);
        let amount = if is_refund { -amount.abs() } else { amount };

        rows.push(Transaction {
            time,
            month: time.format("%Y-%m").to_string(),
            date: time.format("%Y-%m-%d").to_string(),
            direction: value_at(&record, direction_index).unwrap_or_else(|| "/".into()),
            amount,
            category: value_at(&record, category_index).unwrap_or_else(|| "其他".into()),
            description: value_at(&record, description_index).unwrap_or_default(),
            counterparty: value_at(&record, counterparty_index).unwrap_or_else(|| "未知".into()),
            payment_method: value_at(&record, payment_index).unwrap_or_default(),
            status,
            is_refund,
            source: "微信".into(),
        });
    }

    Ok(rows)
}

/// 去掉货币符号与千分位。
fn clean_amount(raw: &str) -> String {
    raw.replace(['¥', ','], "").trim().to_owned()
}

/// 退款判定：状态包含 退款 / 关闭 / 撤销。
fn is_refund_status(status: &str) -> bool {
    let status = status.to_lowercase();
    status.contains("退款") || status.contains("关闭") || status.contains("撤销")
}

fn value_at(record: &csv::StringRecord, index: Option<usize>) -> Option<String> {
    index.and_then(|index| record.get(index)).map(str::to_owned)
}
