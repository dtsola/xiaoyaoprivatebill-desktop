//! P1-1 / P1-2 契约测试：领域模型、数据目录布局、设置读写。

use std::fs;

use bill_core::{DataPaths, Dataset, FileEntry, Settings, Transaction};
use chrono::NaiveDate;

fn sample(name: &str, amount: f64) -> Transaction {
    let time = NaiveDate::from_ymd_opt(2024, 1, 15)
        .expect("日期合法")
        .and_hms_opt(12, 30, 0)
        .expect("时间合法");
    Transaction {
        time,
        month: "2024-01".into(),
        date: "2024-01-15".into(),
        direction: "支出".into(),
        amount,
        category: "餐饮美食".into(),
        description: name.into(),
        counterparty: "星巴克".into(),
        payment_method: "余额宝".into(),
        status: "交易成功".into(),
        is_refund: false,
        source: "支付宝".into(),
    }
}

#[test]
fn ensure_layout_creates_all_directories() {
    let tmp = tempfile::tempdir().expect("临时目录");
    let paths = DataPaths::new(tmp.path().to_path_buf());

    paths.ensure_layout().expect("创建目录布局");

    assert!(paths.bills_dir().is_dir(), "bills/ 必须存在");
    assert!(paths.trash_dir().is_dir(), "trash/ 必须存在");
    assert!(paths.snapshots_dir().is_dir(), "snapshots/ 必须存在");
    assert!(paths.tmp_dir().is_dir(), "tmp/ 必须存在");
    assert!(paths.logs_dir().is_dir(), "logs/ 必须存在");
}

#[test]
fn ensure_layout_is_idempotent() {
    let tmp = tempfile::tempdir().expect("临时目录");
    let paths = DataPaths::new(tmp.path().to_path_buf());

    paths.ensure_layout().expect("第一次创建");
    paths.ensure_layout().expect("第二次不应报错");
}

#[test]
fn settings_fall_back_to_defaults_when_missing() {
    let tmp = tempfile::tempdir().expect("临时目录");
    let paths = DataPaths::new(tmp.path().to_path_buf());

    let settings = Settings::load(&paths.settings_file()).expect("缺失时应返回默认值");

    assert_eq!(settings.schema_version, 1);
    assert!(settings.desktop.close_to_tray);
    assert!(!settings.desktop.close_hint_shown);
    assert!(!settings.mcp.enabled);
    assert_eq!(settings.mcp.port, 17650);
    assert!(settings.mcp.last_error.is_none());
    assert!(settings.data.parity_filenames);
    assert!(settings.data.snapshot_before_clear);
}

#[test]
fn settings_roundtrip_after_save() {
    let tmp = tempfile::tempdir().expect("临时目录");
    let paths = DataPaths::new(tmp.path().to_path_buf());
    paths.ensure_layout().expect("创建目录布局");

    let mut settings = Settings::default();
    settings.mcp.enabled = true;
    settings.desktop.close_to_tray = false;
    settings.save(&paths.settings_file()).expect("保存设置");

    let loaded = Settings::load(&paths.settings_file()).expect("重新读取");
    assert_eq!(loaded, settings);
}

#[test]
fn settings_save_leaves_no_temporary_files() {
    let tmp = tempfile::tempdir().expect("临时目录");
    let paths = DataPaths::new(tmp.path().to_path_buf());
    paths.ensure_layout().expect("创建目录布局");

    Settings::default()
        .save(&paths.settings_file())
        .expect("保存设置");

    let names: Vec<String> = fs::read_dir(tmp.path())
        .expect("读取目录")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    assert!(names.contains(&"settings.json".to_string()));
    assert!(
        !names.iter().any(|name| name.contains(".tmp")),
        "原子写不应残留临时文件: {names:?}"
    );
}

#[test]
fn settings_load_reports_corrupted_json() {
    let tmp = tempfile::tempdir().expect("临时目录");
    let paths = DataPaths::new(tmp.path().to_path_buf());
    paths.ensure_layout().expect("创建目录布局");
    fs::write(paths.settings_file(), b"{ not json").expect("写入损坏内容");

    let error = Settings::load(&paths.settings_file()).expect_err("损坏的 JSON 必须报错");
    assert!(
        error.to_string().contains("设置"),
        "错误信息应说明是设置文件: {error}"
    );
}

#[test]
fn dataset_available_years_is_sorted_unique_desc() {
    let dataset = Dataset {
        transactions: vec![
            sample("a", -10.0),
            Transaction {
                month: "2023-12".into(),
                date: "2023-12-01".into(),
                time: NaiveDate::from_ymd_opt(2023, 12, 1)
                    .expect("日期合法")
                    .and_hms_opt(8, 0, 0)
                    .expect("时间合法"),
                ..sample("b", -20.0)
            },
            Transaction {
                month: "2024-03".into(),
                date: "2024-03-02".into(),
                time: NaiveDate::from_ymd_opt(2024, 3, 2)
                    .expect("日期合法")
                    .and_hms_opt(9, 0, 0)
                    .expect("时间合法"),
                ..sample("c", -30.0)
            },
        ],
        files: Vec::new(),
    };

    assert_eq!(dataset.available_years(), vec![2024, 2023]);
}

#[test]
fn dataset_is_empty_reflects_transactions() {
    let mut dataset = Dataset::default();
    assert!(dataset.is_empty());

    dataset.transactions.push(sample("a", -1.0));
    assert!(!dataset.is_empty());
}

#[test]
fn dataset_files_are_listed_with_source() {
    let mut dataset = Dataset::default();
    dataset.files.push(FileEntry {
        name: "alipay_record_20240101.csv".into(),
        size: 1024,
        source: "支付宝".into(),
        imported_at: "2026-11-11 10:00:00".into(),
    });

    assert_eq!(dataset.file_count(), 1);
    assert_eq!(dataset.files[0].source, "支付宝");
}
