//! 来源探测与文件名清洗，行为基准为 Web 版 `utils/file_utils.py`
//! 与 werkzeug `secure_filename`。

use encoding_rs::UTF_8;
use regex::Regex;
use unicode_normalization::UnicodeNormalization;

/// 账单来源。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// 支付宝。
    Alipay,
    /// 微信。
    Wechat,
    /// 无法识别。
    Unknown,
}

impl Source {
    /// 中文标签，用于文件列表与统计。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Alipay => "支付宝",
            Self::Wechat => "微信",
            Self::Unknown => "未知",
        }
    }
}

/// 探测来源。
///
/// - `.xlsx` 一律视为微信；
/// - `.csv` 读前 1024 字节（调用方传入），UTF-8 解码后含「微信支付账单」为微信，
///   解码失败或其他内容为支付宝；
/// - 其他扩展名返回 [`Source::Unknown`]。
pub fn detect_source(filename: &str, head: &[u8]) -> Source {
    if filename.ends_with(".xlsx") {
        return Source::Wechat;
    }
    if filename.ends_with(".csv") {
        let (text, _, failed) = UTF_8.decode(head);
        if !failed && text.contains("微信支付账单") {
            return Source::Wechat;
        }
        return Source::Alipay;
    }
    Source::Unknown
}

/// 复刻 werkzeug `secure_filename`，并额外去掉微信导出时间戳后缀。
///
/// 顺序：NFKD 归一化 → 仅保留 ASCII → `/` 替换为空格 → 空白切分后 `_` 连接
/// → 删除 `[^A-Za-z0-9_.-]` → 去首尾 `._` → 去掉 `_数字` 时间戳后缀。
pub fn secure_filename(original: &str) -> String {
    let ascii: String = original.nfkd().filter(char::is_ascii).collect();
    let replaced = ascii.replace('/', " ");
    let joined = replaced.split_whitespace().collect::<Vec<_>>().join("_");
    let cleaned = strip_invalid(&joined);
    let trimmed = cleaned.trim_matches(|ch| ch == '.' || ch == '_');
    remove_wechat_timestamp(trimmed)
}

/// 分配最终文件名：清洗 → 空名回退 → 同名追加 `_1`、`_2`。
pub fn allocate_name(
    original: &str,
    exists: impl Fn(&str) -> bool,
    fallback: impl Fn(&str) -> String,
) -> String {
    let mut name = secure_filename(original);
    if name.is_empty() || name.starts_with('.') {
        name = fallback(extension_of(original));
    }

    if !exists(&name) {
        return name;
    }

    let (base, ext) = split_extension(&name);
    let mut counter = 1;
    loop {
        let candidate = format!("{base}_{counter}{ext}");
        if !exists(&candidate) {
            return candidate;
        }
        counter += 1;
    }
}

fn strip_invalid(value: &str) -> String {
    static INVALID: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let pattern = INVALID.get_or_init(|| Regex::new(r"[^A-Za-z0-9_.-]").expect("正则合法"));
    pattern.replace_all(value, "").into_owned()
}

/// `20240101-20240131_20251216085838.xlsx` → `20240101-20240131.xlsx`。
fn remove_wechat_timestamp(name: &str) -> String {
    static TIMESTAMP: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let pattern = TIMESTAMP
        .get_or_init(|| Regex::new(r"^(\d{8}-\d{8})_\d+(\.xlsx|\.csv)$").expect("正则合法"));
    if let Some(captures) = pattern.captures(name) {
        return format!("{}{}", &captures[1], &captures[2]);
    }
    name.to_owned()
}

/// 等价于 `os.path.splitext`：仅当点在非首位时切分。
fn split_extension(name: &str) -> (&str, &str) {
    match name.rfind('.') {
        Some(index) if index > 0 => (&name[..index], &name[index..]),
        _ => (name, ""),
    }
}

fn extension_of(name: &str) -> &str {
    split_extension(name).1
}
