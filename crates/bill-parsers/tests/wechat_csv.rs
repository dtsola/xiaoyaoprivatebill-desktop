//! P1-4 契约测试：微信 CSV 解析。

use bill_parsers::wechat::parse_wechat_csv;
use chrono::NaiveDate;

const FIXTURE: &str = "微信支付账单明细\n\
微信昵称：[小遥]\n\
起始时间：[2024-01-01 00:00:00] 终止时间：[2024-01-31 23:59:59]\n\
导出类型：[全部]\n\
导出时间：[2024-02-01 10:00:00]\n\
\n\
微信支付账单明细列表\n\
交易时间,交易类型,交易对方,商品,收/支,金额(元),支付方式,当前状态,交易单号,商户单号,备注\n\
2024-01-15 12:30:00,商户消费,星巴克,咖啡,支出,¥28.50,零钱,支付成功,1001,M001,\n\
2024-01-16 08:20:00,交通出行,地铁公司,地铁票,支出,¥6.00,零钱通,支付成功,1002,M002,\n\
2024-01-17 19:00:00,转账,张三,转账收款,收入,¥100.00,零钱,已存入零钱,1003,M003,\n\
----------------------微信支付账单明细列表--------------------\n";

fn parse(text: &str) -> Vec<bill_core::Transaction> {
    parse_wechat_csv(text.as_bytes()).expect("解析应成功")
}

#[test]
fn parses_fixture_and_maps_columns() {
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
    assert!((first.amount - 28.50).abs() < 1e-9, "金额应去掉 ¥");
    assert_eq!(first.category, "商户消费", "交易类型 → 交易分类");
    assert_eq!(first.description, "咖啡", "商品 → 商品说明");
    assert_eq!(first.counterparty, "星巴克");
    assert_eq!(first.payment_method, "零钱", "支付方式 → 收/付款方式");
    assert_eq!(first.status, "支付成功", "当前状态 → 交易状态");
    assert!(!first.is_refund);
    assert_eq!(first.source, "微信");
}

#[test]
fn strips_thousands_separator() {
    // 含千分位的金额在 CSV 中必须带引号，否则逗号会被当作分隔符。
    let text = FIXTURE.replace("¥28.50", "\"¥1,280.50\"");
    let rows = parse(&text);
    assert!((rows[0].amount - 1280.50).abs() < 1e-9);
}

#[test]
fn refund_rows_become_negative() {
    let text = FIXTURE.replace("支付成功,1001", "已退款,1001");
    let rows = parse(&text);

    assert!(rows[0].is_refund, "已退款应标记为退款");
    assert!(
        (rows[0].amount + 28.50).abs() < 1e-9,
        "退款金额应为负绝对值"
    );
}

#[test]
fn closed_and_reversed_rows_are_refunds() {
    let closed = parse(&FIXTURE.replace("支付成功,1002", "交易关闭,1002"));
    assert!(closed[1].is_refund);

    let reversed = parse(&FIXTURE.replace("支付成功,1002", "已撤销,1002"));
    assert!(reversed[1].is_refund);
}

#[test]
fn handles_utf8_bom() {
    let with_bom = format!("\u{feff}{FIXTURE}");
    let rows = parse(&with_bom);
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].counterparty, "星巴克");
}

#[test]
fn errors_when_header_missing() {
    let error = parse_wechat_csv(b"a,b\n1,2\n").expect_err("缺少表头必须报错");
    assert!(
        error.to_string().contains("无法找到微信账单表头行"),
        "错误信息应可读: {error}"
    );
}

#[test]
fn errors_when_amount_column_missing() {
    let text = "交易时间,交易类型,交易对方,商品,收/支,支付方式,当前状态\n\
2024-01-15 12:30:00,商户消费,星巴克,咖啡,支出,零钱,支付成功\n";

    let error = parse_wechat_csv(text.as_bytes()).expect_err("缺少金额列必须报错");
    assert!(
        error.to_string().contains("金额"),
        "错误信息应指出缺失字段: {error}"
    );
}

#[test]
fn fills_defaults_for_missing_optional_columns() {
    let text = "交易时间,交易类型,金额(元)\n\
2024-01-15 12:30:00,商户消费,28.50\n";

    let rows = parse_wechat_csv(text.as_bytes()).expect("可选列缺失应补默认值");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].counterparty, "未知");
    assert_eq!(rows[0].direction, "/");
    assert_eq!(rows[0].description, "");
    assert_eq!(rows[0].payment_method, "");
}

#[test]
fn skips_rows_with_unparseable_time_or_amount() {
    let text = "交易时间,交易类型,金额(元)\n\
不是时间,商户消费,1.00\n\
2024-01-15 12:30:00,商户消费,不是金额\n\
2024-01-16 09:00:00,商户消费,2.00\n";

    let rows = parse_wechat_csv(text.as_bytes()).expect("非法行应跳过");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].date, "2024-01-16");
}
