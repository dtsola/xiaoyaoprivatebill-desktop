//! P1-7 契约测试：目录扫描、合并、稳定排序与演示数据。

use std::fs;

use bill_services::dataset::{demo_dataset, load_dataset, ServiceError};

const ALIPAY: &str = "交易时间,交易分类,交易对方,商品说明,收/支,金额,收/付款方式,交易状态\n\
2024-01-15 12:30:00,餐饮美食,星巴克,咖啡,支出,28.50,余额宝,交易成功\n\
2024-01-10 09:00:00,交通出行,地铁,地铁票,支出,6.00,余额宝,交易成功\n";

const WECHAT: &str = "微信支付账单明细\n\
微信昵称：[小遥]\n\
\n\
交易时间,交易类型,交易对方,商品,收/支,金额(元),支付方式,当前状态\n\
2024-01-12 08:00:00,商户消费,便利店,早餐,支出,¥5.00,零钱,支付成功\n";

fn write(dir: &std::path::Path, name: &str, content: &str) {
    fs::write(dir.join(name), content).expect("写入夹具");
}

#[test]
fn scans_merges_and_sorts_by_time() {
    let tmp = tempfile::tempdir().expect("临时目录");
    write(tmp.path(), "alipay.csv", ALIPAY);
    write(tmp.path(), "wechat.csv", WECHAT);

    let dataset = load_dataset(tmp.path()).expect("加载成功");

    let descriptions: Vec<&str> = dataset
        .transactions
        .iter()
        .map(|row| row.description.as_str())
        .collect();
    assert_eq!(descriptions, vec!["地铁票", "早餐", "咖啡"]);
    assert_eq!(dataset.file_count(), 2);
    assert_eq!(dataset.available_years(), vec![2024]);
}

#[test]
fn source_is_recorded_per_file() {
    let tmp = tempfile::tempdir().expect("临时目录");
    write(tmp.path(), "alipay.csv", ALIPAY);
    write(tmp.path(), "wechat.csv", WECHAT);

    let dataset = load_dataset(tmp.path()).expect("加载成功");
    let mut sources: Vec<&str> = dataset
        .files
        .iter()
        .map(|file| file.source.as_str())
        .collect();
    sources.sort_unstable();
    assert_eq!(sources, vec!["微信", "支付宝"]);
}

#[test]
fn equal_times_keep_file_name_order_and_row_order() {
    let tmp = tempfile::tempdir().expect("临时目录");
    write(
        tmp.path(),
        "b.csv",
        "交易时间,金额,商品说明\n2024-01-01 10:00:00,2.00,b1\n",
    );
    write(
        tmp.path(),
        "a.csv",
        "交易时间,金额,商品说明\n2024-01-01 10:00:00,1.00,a1\n2024-01-01 10:00:00,1.00,a2\n",
    );

    let dataset = load_dataset(tmp.path()).expect("加载成功");
    let descriptions: Vec<&str> = dataset
        .transactions
        .iter()
        .map(|row| row.description.as_str())
        .collect();
    assert_eq!(descriptions, vec!["a1", "a2", "b1"], "排序必须稳定");
}

#[test]
fn skips_unparseable_files_and_ignores_other_extensions() {
    let tmp = tempfile::tempdir().expect("临时目录");
    write(tmp.path(), "broken.csv", "not a bill");
    write(tmp.path(), "readme.txt", "ignore me");
    write(tmp.path(), "alipay.csv", ALIPAY);

    let dataset = load_dataset(tmp.path()).expect("仍应加载成功");
    assert_eq!(dataset.file_count(), 1);
    assert_eq!(dataset.transactions.len(), 2);
}

#[test]
fn errors_when_no_bill_can_be_loaded() {
    let tmp = tempfile::tempdir().expect("临时目录");
    write(tmp.path(), "readme.txt", "nothing");

    let error = load_dataset(tmp.path()).expect_err("没有账单必须报错");
    match error {
        ServiceError::NoData(message) => {
            assert!(message.contains("未找到任何支付宝"), "信息: {message}");
        }
        other => panic!("错误类型不符: {other:?}"),
    }
}

#[test]
fn demo_dataset_has_7396_rows() {
    let dataset = demo_dataset().expect("演示数据应可解析");
    assert_eq!(dataset.transactions.len(), 7396);
    // 演示数据按文件原序，不排序：首行为 2024-12-31。
    assert_eq!(dataset.transactions[0].date, "2024-12-31");
    assert_eq!(dataset.transactions[0].category, "收入");
    assert_eq!(dataset.transactions[0].source, "支付宝");
}
