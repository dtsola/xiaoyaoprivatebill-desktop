//! 账单文件管理：导入、列表、删除（进回收站）、清空。
//!
//! 行为基准为 Web 版 `api/files.py`；删除与清空一律移入 `trash/`，永不自动清理。

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use bill_core::{write_atomic, CoreError, DataPaths, FileEntry};
use bill_parsers::filename::{allocate_name, detect_source};
use chrono::Local;
use serde_json::json;

/// 单个文件大小上限 16MB。
pub const MAX_UPLOAD_SIZE: u64 = 16 * 1024 * 1024;

static TRASH_COUNTER: AtomicU64 = AtomicU64::new(0);

/// 文件管理错误。
#[derive(Debug, thiserror::Error)]
pub enum FileError {
    /// 不支持的文件类型。
    #[error("不支持的文件类型")]
    Unsupported,
    /// 文件超过大小上限。
    #[error("文件超过 16MB 上限: {0}")]
    TooLarge(String),
    /// 文件名非法（路径穿越、分隔符等）。
    #[error("无效的文件名")]
    InvalidName,
    /// 文件不存在。
    #[error("文件不存在")]
    NotFound,
    /// 没有可导入的文件。
    #[error("没有文件被上传")]
    Empty,
    /// 读写失败。
    #[error("文件操作失败: {0}")]
    Io(String),
}

/// 导入文件，默认 16MB 上限。
pub fn import_paths(paths: &DataPaths, sources: &[PathBuf]) -> Result<Vec<String>, FileError> {
    import_paths_with_limit(paths, sources, MAX_UPLOAD_SIZE)
}

/// 导入文件，可指定大小上限（测试用）。
pub fn import_paths_with_limit(
    paths: &DataPaths,
    sources: &[PathBuf],
    limit: u64,
) -> Result<Vec<String>, FileError> {
    if sources.is_empty() {
        return Err(FileError::Empty);
    }

    let mut imported = Vec::new();
    for source in sources {
        let name = source
            .file_name()
            .map(|value| value.to_string_lossy().into_owned())
            .ok_or(FileError::InvalidName)?;
        if !has_bill_extension(&name) {
            return Err(FileError::Unsupported);
        }

        let bytes = fs::read(source)
            .map_err(|error| FileError::Io(format!("读取 {} 失败: {error}", source.display())))?;
        if bytes.len() as u64 > limit {
            return Err(FileError::TooLarge(name));
        }

        let final_name = allocate_name(
            &name,
            |candidate| paths.bills_dir().join(candidate).exists(),
            fallback_name,
        );
        write_atomic(&paths.bills_dir().join(&final_name), &bytes)?;
        imported.push(final_name);
    }

    // 首次导入成功后激活本地会话（演示会话保持不变）。
    bill_core::Session::mark_import(paths)?;

    Ok(imported)
}

/// 列出 `bills/` 中的账单文件，按文件名升序。
pub fn list_files(paths: &DataPaths) -> Result<Vec<FileEntry>, FileError> {
    let mut files = Vec::new();
    let entries = match fs::read_dir(paths.bills_dir()) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(files),
        Err(error) => return Err(FileError::Io(error.to_string())),
    };

    for entry in entries.filter_map(|entry| entry.ok()) {
        let path = entry.path();
        let Some(name) = path
            .file_name()
            .map(|value| value.to_string_lossy().into_owned())
        else {
            continue;
        };
        if !path.is_file() || !has_bill_extension(&name) {
            continue;
        }

        let Ok(bytes) = fs::read(&path) else {
            continue;
        };
        let head = &bytes[..bytes.len().min(1024)];
        files.push(FileEntry {
            name,
            size: bytes.len() as u64,
            source: detect_source(&path.to_string_lossy(), head)
                .as_str()
                .to_owned(),
            imported_at: modified_at(&path),
        });
    }

    files.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(files)
}

/// 把文件移入回收站并写 manifest。
pub fn delete_file(paths: &DataPaths, filename: &str) -> Result<(), FileError> {
    if !has_bill_extension(filename) || !is_plain_name(filename) {
        return Err(FileError::InvalidName);
    }

    let source = paths.bills_dir().join(filename);
    if !source.is_file() {
        return Err(FileError::NotFound);
    }

    let trash = create_trash_entry(paths, "delete", &[filename.to_owned()])?;
    fs::rename(&source, trash.join("files").join(filename))
        .map_err(|error| FileError::Io(format!("移动到回收站失败: {error}")))?;
    Ok(())
}

/// 清空 `bills/`，所有文件移入同一个回收站条目，返回移动数量。
pub fn clear_data(paths: &DataPaths) -> Result<usize, FileError> {
    let entries = match fs::read_dir(paths.bills_dir()) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(error) => return Err(FileError::Io(error.to_string())),
    };

    let names: Vec<String> = entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_file())
        .filter_map(|entry| entry.file_name().to_str().map(str::to_owned))
        .filter(|name| has_bill_extension(name))
        .collect();

    if names.is_empty() {
        return Ok(0);
    }

    let trash = create_trash_entry(paths, "clear", &names)?;
    for name in &names {
        fs::rename(paths.bills_dir().join(name), trash.join("files").join(name))
            .map_err(|error| FileError::Io(format!("移动到回收站失败: {error}")))?;
    }
    Ok(names.len())
}

fn create_trash_entry(
    paths: &DataPaths,
    intent: &str,
    names: &[String],
) -> Result<PathBuf, FileError> {
    let seq = TRASH_COUNTER.fetch_add(1, Ordering::Relaxed);
    let id = format!("{}-{seq:04}", Local::now().format("%Y%m%d%H%M%S%3f"));
    let dir = paths.trash_dir().join(&id);
    fs::create_dir_all(dir.join("files"))
        .map_err(|error| FileError::Io(format!("创建回收站条目失败: {error}")))?;

    let manifest = json!({
        "intent": intent,
        "created_at": Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        "files": names,
    });
    let bytes = serde_json::to_vec_pretty(&manifest)
        .map_err(|error| FileError::Io(format!("序列化 manifest 失败: {error}")))?;
    write_atomic(&dir.join("manifest.json"), &bytes).map_err(FileError::from)?;
    Ok(dir)
}

fn has_bill_extension(name: &str) -> bool {
    name.ends_with(".csv") || name.ends_with(".xlsx")
}

fn is_plain_name(name: &str) -> bool {
    !name.is_empty()
        && !name.contains('/')
        && !name.contains('\\')
        && !name.contains("..")
        && !name.contains('\0')
}

fn fallback_name(extension: &str) -> String {
    let seq = TRASH_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!(
        "upload_{}_{seq:04}{extension}",
        Local::now().format("%Y%m%d%H%M%S")
    )
}

fn modified_at(path: &Path) -> String {
    fs::metadata(path)
        .and_then(|meta| meta.modified())
        .ok()
        .map(|time| {
            let local: chrono::DateTime<Local> = time.into();
            local.format("%Y-%m-%d %H:%M:%S").to_string()
        })
        .unwrap_or_default()
}

impl From<CoreError> for FileError {
    fn from(error: CoreError) -> Self {
        FileError::Io(error.to_string())
    }
}
