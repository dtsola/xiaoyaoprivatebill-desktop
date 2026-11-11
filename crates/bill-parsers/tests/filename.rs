//! P1-6 契约测试：来源探测与文件名清洗。
//!
//! 行为基准为 Web 版 `utils/file_utils.py` 与 werkzeug `secure_filename`。

use bill_parsers::filename::{allocate_name, detect_source, secure_filename, Source};

#[test]
fn keeps_plain_ascii_name() {
    assert_eq!(
        secure_filename("alipay_record_2026.csv"),
        "alipay_record_2026.csv"
    );
}

#[test]
fn drops_non_ascii_and_punctuation() {
    // 复刻 D3：纯中文文件名会退化为扩展名。
    assert_eq!(secure_filename("支付宝账单.csv"), "csv");
    assert_eq!(
        secure_filename("微信支付账单(20240101-20240131).csv"),
        "20240101-20240131.csv"
    );
}

#[test]
fn joins_whitespace_with_underscore_then_strips() {
    assert_eq!(secure_filename("报告 2024.csv"), "2024.csv");
    assert_eq!(secure_filename("  a   b.csv  "), "a_b.csv");
}

#[test]
fn neutralizes_path_separators() {
    // werkzeug 在 Linux 上运行，只把 "/" 替换为空格；反斜杠被字符过滤删除。
    assert_eq!(secure_filename("/etc/passwd.csv"), "etc_passwd.csv");
    assert_eq!(secure_filename("a\\b.csv"), "ab.csv");
    assert_eq!(secure_filename("../../evil.csv"), "evil.csv");
}

#[test]
fn strips_leading_and_trailing_dots_and_underscores() {
    assert_eq!(secure_filename("...name...csv"), "name...csv");
}

#[test]
fn removes_wechat_export_timestamp_suffix() {
    assert_eq!(
        secure_filename("20240101-20240131_20251216085838.xlsx"),
        "20240101-20240131.xlsx"
    );
    assert_eq!(
        secure_filename("20240101-20240131_20251216085838.csv"),
        "20240101-20240131.csv"
    );
    // 大小写不同不匹配正则，保持原样。
    assert_eq!(
        secure_filename("20240101-20240131_20251216085838.XLSX"),
        "20240101-20240131_20251216085838.XLSX"
    );
}

#[test]
fn falls_back_when_name_is_empty() {
    let name = allocate_name(
        "账单",
        |_| false,
        |ext| format!("upload_0000000000_deadbeef{ext}"),
    );
    assert_eq!(name, "upload_0000000000_deadbeef");
}

#[test]
fn fallback_name_also_avoids_conflicts() {
    let name = allocate_name(
        "账单",
        |candidate| candidate == "upload_0000000000_deadbeef",
        |ext| format!("upload_0000000000_deadbeef{ext}"),
    );
    assert_eq!(name, "upload_0000000000_deadbeef_1");
}

#[test]
fn appends_counter_for_duplicates() {
    let name = allocate_name(
        "alipay.csv",
        |candidate| candidate == "alipay.csv" || candidate == "alipay_1.csv",
        |ext| format!("upload_x{ext}"),
    );
    assert_eq!(name, "alipay_2.csv");
}

#[test]
fn allocates_first_available_name_when_no_conflict() {
    let name = allocate_name("alipay.csv", |_| false, |ext| format!("upload_x{ext}"));
    assert_eq!(name, "alipay.csv");
}

#[test]
fn detects_xlsx_as_wechat() {
    assert_eq!(detect_source("2024.xlsx", b"anything"), Source::Wechat);
}

#[test]
fn detects_wechat_csv_by_header_bytes() {
    let head = "微信支付账单明细\n微信昵称：[小遥]\n".as_bytes();
    assert_eq!(detect_source("bill.csv", head), Source::Wechat);
}

#[test]
fn detects_alipay_csv_by_default() {
    let head = "支付宝交易记录明细查询\n".as_bytes();
    assert_eq!(detect_source("bill.csv", head), Source::Alipay);
}

#[test]
fn detects_alipay_when_bytes_are_not_utf8() {
    let head = &[0xFF, 0xFE, 0x00, 0x41];
    assert_eq!(detect_source("bill.csv", head), Source::Alipay);
}

#[test]
fn returns_unknown_for_other_extensions() {
    assert_eq!(
        detect_source("bill.txt", "微信支付账单".as_bytes()),
        Source::Unknown
    );
}

#[test]
fn source_serializes_to_chinese_label() {
    assert_eq!(Source::Alipay.as_str(), "支付宝");
    assert_eq!(Source::Wechat.as_str(), "微信");
    assert_eq!(Source::Unknown.as_str(), "未知");
}
