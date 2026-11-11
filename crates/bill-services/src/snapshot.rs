//! 快照：清空前把 `bills/` 复制到 `snapshots/<id>/`，永不自建清理。
//!
//! 失败即拒绝后续清空（fail-closed），见技术方案 03 §8.4。

use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

use bill_core::{write_atomic, DataPaths};
use chrono::Local;
use serde_json::json;

use crate::files::FileError;

static SNAPSHOT_COUNTER: AtomicU64 = AtomicU64::new(0);

/// 创建快照，返回 `(快照 ID, 文件数)`。
pub fn create_snapshot(paths: &DataPaths) -> Result<(String, usize), FileError> {
    let entries = match fs::read_dir(paths.bills_dir()) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return create_entry(paths, &[]);
        }
        Err(error) => return Err(FileError::Io(error.to_string())),
    };

    let names: Vec<String> = entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_file())
        .filter_map(|entry| entry.file_name().to_str().map(str::to_owned))
        .filter(|name| name.ends_with(".csv") || name.ends_with(".xlsx"))
        .collect();

    create_entry(paths, &names)
}

fn create_entry(paths: &DataPaths, names: &[String]) -> Result<(String, usize), FileError> {
    let seq = SNAPSHOT_COUNTER.fetch_add(1, Ordering::Relaxed);
    let id = format!("{}-{seq:04}", Local::now().format("%Y%m%d%H%M%S%3f"));
    let dir = paths.snapshots_dir().join(&id);
    let files_dir = dir.join("files");
    fs::create_dir_all(&files_dir)
        .map_err(|error| FileError::Io(format!("创建快照目录失败: {error}")))?;

    let mut copied = Vec::new();
    for name in names {
        let source = paths.bills_dir().join(name);
        let bytes = fs::read(&source)
            .map_err(|error| FileError::Io(format!("读取 {name} 失败: {error}")))?;
        let target = files_dir.join(name);
        fs::write(&target, &bytes)
            .map_err(|error| FileError::Io(format!("写入快照 {name} 失败: {error}")))?;

        // 校验：复制后大小必须一致，否则拒绝清空。
        let copied_size = fs::metadata(&target)
            .map_err(|error| FileError::Io(format!("校验快照 {name} 失败: {error}")))?
            .len();
        if copied_size != bytes.len() as u64 {
            return Err(FileError::Io(format!("快照校验失败: {name}")));
        }
        copied.push(json!({ "name": name, "size": bytes.len() }));
    }

    let manifest = json!({
        "created_at": Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        "files": copied,
    });
    let bytes = serde_json::to_vec_pretty(&manifest)
        .map_err(|error| FileError::Io(format!("序列化快照 manifest 失败: {error}")))?;
    write_atomic(&dir.join("manifest.json"), &bytes)?;

    Ok((id, names.len()))
}
