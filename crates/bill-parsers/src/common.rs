//! 解析器共用工具：解码、行定位、字段取值、时间解析。

use chrono::NaiveDateTime;
use encoding_rs::{GBK, UTF_8};

use crate::error::ParseError;

const TIME_FORMATS: [&str; 5] = [
    "%Y-%m-%d %H:%M:%S",
    "%Y-%m-%d %H:%M",
    "%Y-%m-%d",
    "%Y/%m/%d %H:%M:%S",
    "%Y/%m/%d %H:%M",
];

/// 支付宝：先尝试 UTF-8，失败再按 GBK 解码。
///
/// 与 Python 的「GBK 优先」顺序相反：WHATWG GBK（encoding_rs）会把部分 UTF-8
/// 字节序列当作合法 GBK 并解出乱码，而 Python 的 gbk codec 会直接报错。
/// 真实支付宝导出是 GBK，不可能是合法 UTF-8，因此 UTF-8 优先不会误判。
pub(crate) fn decode_gbk_or_utf8(bytes: &[u8]) -> Result<String, ParseError> {
    if let Ok(text) = std::str::from_utf8(bytes) {
        return Ok(text.trim_start_matches('\u{feff}').to_owned());
    }
    let (gbk_text, _, gbk_failed) = GBK.decode(bytes);
    if !gbk_failed {
        return Ok(gbk_text.into_owned());
    }
    Err(ParseError::Decode)
}

/// 微信：UTF-8（自动去除 BOM）。
pub(crate) fn decode_utf8_sig(bytes: &[u8]) -> Result<String, ParseError> {
    let (text, _, failed) = UTF_8.decode(bytes);
    if failed {
        return Err(ParseError::Decode);
    }
    Ok(text.into_owned())
}

/// 找到首个满足条件的整行字节偏移。
pub(crate) fn line_offset_matching(text: &str, predicate: impl Fn(&str) -> bool) -> Option<usize> {
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        if predicate(line) {
            return Some(offset);
        }
        offset += line.len();
    }
    None
}

/// 找到首个包含关键字整行的字节偏移。
pub(crate) fn line_offset(text: &str, needle: &str) -> Option<usize> {
    line_offset_matching(text, |line| line.contains(needle))
}

/// 把一行按 CSV 解析后取第一个字段。
pub(crate) fn first_field(line: &str) -> Option<String> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(line.as_bytes());
    reader
        .records()
        .next()
        .and_then(|record| record.ok())
        .and_then(|record| record.get(0).map(str::to_owned))
}

/// 按列名取值；列不存在时返回 `None`。
pub(crate) fn text_field(
    record: &csv::StringRecord,
    headers: &csv::StringRecord,
    name: &str,
) -> Option<String> {
    headers
        .iter()
        .position(|header| header == name)
        .and_then(|index| record.get(index))
        .map(str::to_owned)
}

/// 按候选列名依次查找列下标，用于兼容同一字段的多种列名。
pub(crate) fn header_index(headers: &csv::StringRecord, names: &[&str]) -> Option<usize> {
    names
        .iter()
        .find_map(|name| headers.iter().position(|header| header == *name))
}

/// 按固定格式依次尝试解析时间。
pub(crate) fn parse_time(raw: &str) -> Option<NaiveDateTime> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    TIME_FORMATS
        .iter()
        .find_map(|format| NaiveDateTime::parse_from_str(raw, format).ok())
}
