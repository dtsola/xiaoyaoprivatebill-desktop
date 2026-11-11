//! P1-3 契约测试：支付宝 CSV 解析。
//!
//! 行为基准为 Web 版 `parsers/alipay.py`，包含故意保留的缺陷 D1 / D2。

use bill_parsers::alipay::parse_alipay_csv;
use chrono::NaiveDate;

const FIXTURE: &str = "支付宝交易记录明细查询\n\
账号:[test@example.com]\n\
起始日期:[2024-01-01 00:00:00]    终止日期:[2024-01-31 23:59:59]\n\
---------------------------------交易记录明细列表------------------------------------\n\
交易时间,交易分类,交易对方,对方账号,商品说明,收/支,金额,收/付款方式,交易状态,交易订单号,商家订单号,备注\n\
2024-01-15 12:30:00,餐饮美食,星巴克,starbucks@example.com,购买咖啡,支出,28.50,余额宝,交易成功,20240115001,SH001,\n\
2024-01-16 08:20:00,交通出行,地铁公司,metro@example.com,地铁票,支出,6.00,余额宝,交易成功,20240116001,SH002,\n\
2024-01-17 19:00:00,转账,张三,zhangsan@example.com,转账收款,收入,100.00,余额宝,交易成功,20240117001,SH003,\n\
------------------------------------------------------------------------------------\n\
导出时间:[2024-02-01 10:00:00]\n";

fn parse(text: &str) -> Vec<bill_core::Transaction> {
    parse_alipay_csv(text.as_bytes()).expect("解析应成功")
}

#[test]
fn parses_utf8_fixture() {
    let rows = parse(FIXTURE);
    assert_eq!(rows.len(), 3, "尾部说明行不应产生交易");

    let first = &rows[0];
    assert_eq!(
        first.time,
        NaiveDate::from_ymd_opt(2024, 1, 15)
            .expect("日期合法")
            .and_hms_opt(12, 30, 0)
            .expect("时间合法")
    );
    assert_eq!(first.month, "2024-01");
    assert_eq!(first.date, "2024-01-15");
    assert_eq!(first.direction, "支出");
    assert!((first.amount - 28.50).abs() < 1e-9);
    assert_eq!(first.category, "餐饮美食");
    assert_eq!(first.description, "购买咖啡");
    assert_eq!(first.counterparty, "星巴克");
    assert_eq!(first.payment_method, "余额宝");
    assert_eq!(first.status, "交易成功");
    assert!(!first.is_refund, "D1：退款判定应恒为 false");
    assert_eq!(first.source, "支付宝");
}

#[test]
fn preserves_file_order() {
    let rows = parse(FIXTURE);
    let descriptions: Vec<&str> = rows.iter().map(|row| row.description.as_str()).collect();
    assert_eq!(descriptions, vec!["购买咖啡", "地铁票", "转账收款"]);
}

#[test]
fn parses_gbk_fixture() {
    let (bytes, _, had_errors) = encoding_rs::GBK.encode(FIXTURE);
    assert!(!had_errors, "样例应可编码为 GBK");

    let rows = parse_alipay_csv(&bytes).expect("GBK 文件应解析成功");
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].counterparty, "星巴克");
}

#[test]
fn falls_back_to_utf8_when_gbk_fails() {
    // 直接把 UTF-8 字节交给解析器：GBK 解码会在多字节中文处出错，应回退 UTF-8。
    let text = FIXTURE.replace("星巴克", "Müller 咖啡");

    let rows = parse_alipay_csv(text.as_bytes()).expect("UTF-8 回退应成功");
    assert_eq!(rows[0].counterparty, "Müller 咖啡");
}

#[test]
fn errors_when_header_missing() {
    let error = parse_alipay_csv(b"a,b\n1,2\n").expect_err("缺少表头必须报错");
    assert!(
        error.to_string().contains("无法找到支付宝账单表头行"),
        "错误信息应可读: {error}"
    );
}

#[test]
fn errors_when_amount_column_missing() {
    // D2：不把「金额（元）」映射为「金额」，按 Python 行为报错。
    let text = "交易时间,交易分类,交易对方,商品说明,收/支,金额（元）,交易状态\n\
2024-01-15 12:30:00,餐饮美食,星巴克,购买咖啡,支出,28.50,交易成功\n";

    let error = parse_alipay_csv(text.as_bytes()).expect_err("缺少金额列必须报错");
    assert!(
        error.to_string().contains("金额"),
        "错误信息应指出缺失字段: {error}"
    );
}

#[test]
fn fills_defaults_for_missing_optional_columns() {
    let text = "交易时间,金额,交易状态\n\
2024-01-15 12:30:00,28.50,交易成功\n";

    let rows = parse_alipay_csv(text.as_bytes()).expect("可选列缺失应补默认值");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].counterparty, "未知");
    assert_eq!(rows[0].direction, "/");
    assert_eq!(rows[0].description, "");
    assert_eq!(rows[0].category, "其他");
    assert_eq!(rows[0].payment_method, "");
}

#[test]
fn skips_rows_with_unparseable_time() {
    let text = "交易时间,金额,交易状态\n\
不是时间,1.00,交易成功\n\
2024-01-15 12:30:00,28.50,交易成功\n";

    let rows = parse_alipay_csv(text.as_bytes()).expect("非法时间行应跳过而不是整体失败");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].date, "2024-01-15");
}
