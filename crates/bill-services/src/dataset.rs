//! 数据集：扫描 `bills/`、合并解析结果、稳定排序；演示数据内嵌。
//!
//! 行为基准为 Web 版 `services/data_loader.py`。

use std::fs;
use std::path::{Path, PathBuf};

use bill_core::{Dataset, FileEntry, Transaction};
use bill_parsers::alipay::parse_alipay_csv;
use bill_parsers::filename::{detect_source, Source};
use bill_parsers::wechat::{parse_wechat_csv, parse_wechat_xlsx};
use chrono::{DateTime, Local};

/// 业务层错误。
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    /// 没有任何账单可以被加载。
    #[error("{0}")]
    NoData(String),
    /// 目录读取失败。
    #[error("读取账单目录失败: {0}")]
    Io(String),
}

/// 扫描目录并加载数据集。
///
/// - 只处理 `.csv` / `.xlsx`，按文件名升序读取；
/// - 单个文件解析失败时跳过，不影响其他文件；
/// - 合并后按 `time` 升序稳定排序（同一时间保持文件顺序与行序）；
/// - 没有任何文件解析成功时报 [`ServiceError::NoData`]。
pub fn load_dataset(bills_dir: &Path) -> Result<Dataset, ServiceError> {
    let mut paths = bill_paths(bills_dir)?;

    let mut transactions = Vec::new();
    let mut files = Vec::new();
    for path in paths.drain(..) {
        let Some(name) = path
            .file_name()
            .map(|value| value.to_string_lossy().into_owned())
        else {
            continue;
        };
        let Ok(bytes) = fs::read(&path) else {
            continue;
        };

        let head = &bytes[..bytes.len().min(1024)];
        let source = detect_source(&name, head);
        let parsed = match source {
            Source::Wechat if name.ends_with(".xlsx") => parse_wechat_xlsx(&bytes),
            Source::Wechat => parse_wechat_csv(&bytes),
            Source::Alipay => parse_alipay_csv(&bytes),
            Source::Unknown => continue,
        };
        let Ok(rows) = parsed else {
            continue;
        };

        transactions.extend(rows);
        files.push(FileEntry {
            name,
            size: bytes.len() as u64,
            source: source.as_str().to_owned(),
            imported_at: modified_at(&path),
        });
    }

    if files.is_empty() {
        return Err(ServiceError::NoData(
            "未找到任何支付宝(.csv)或微信(.xlsx)账单文件".to_owned(),
        ));
    }

    // sort_by_key 是稳定排序，同一时间保持文件顺序与行序。
    transactions.sort_by_key(|transaction| transaction.time);
    Ok(Dataset {
        transactions,
        files,
    })
}

/// 内嵌演示数据（7,396 条），按文件原序，不排序。
pub fn demo_dataset() -> Result<Dataset, ServiceError> {
    let text = include_str!("../data/sample_data.csv");
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(text.as_bytes());

    let mut transactions = Vec::new();
    for record in reader.records() {
        let Ok(record) = record else {
            continue;
        };
        let Some(time) = record.get(0).and_then(|raw| {
            chrono::NaiveDateTime::parse_from_str(raw.trim(), "%Y-%m-%d %H:%M:%S").ok()
        }) else {
            continue;
        };

        transactions.push(Transaction {
            time,
            month: time.format("%Y-%m").to_string(),
            date: time.format("%Y-%m-%d").to_string(),
            direction: field(&record, 4),
            amount: field(&record, 5).parse().unwrap_or_default(),
            category: field(&record, 1),
            description: field(&record, 3),
            counterparty: field(&record, 2),
            payment_method: field(&record, 6),
            status: field(&record, 7),
            is_refund: field(&record, 9).eq_ignore_ascii_case("true"),
            source: field(&record, 8),
        });
    }

    Ok(Dataset {
        transactions,
        files: vec![FileEntry {
            name: "sample_data.csv".to_owned(),
            size: text.len() as u64,
            source: "演示数据".to_owned(),
            imported_at: String::new(),
        }],
    })
}

/// 列出可解析的账单文件，按文件名升序。
fn bill_paths(bills_dir: &Path) -> Result<Vec<PathBuf>, ServiceError> {
    let entries = fs::read_dir(bills_dir)
        .map_err(|error| ServiceError::Io(format!("{}: {error}", bills_dir.display())))?;

    let mut paths: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path
                    .extension()
                    .and_then(|value| value.to_str())
                    .is_some_and(|ext| ext == "csv" || ext == "xlsx")
        })
        .collect();
    paths.sort();
    Ok(paths)
}

fn field(record: &csv::StringRecord, index: usize) -> String {
    record.get(index).unwrap_or_default().to_owned()
}

fn modified_at(path: &Path) -> String {
    fs::metadata(path)
        .and_then(|meta| meta.modified())
        .ok()
        .map(|time| {
            let local: DateTime<Local> = time.into();
            local.format("%Y-%m-%d %H:%M:%S").to_string()
        })
        .unwrap_or_default()
}
