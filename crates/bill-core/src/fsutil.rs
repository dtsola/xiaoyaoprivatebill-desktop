//! 文件系统工具。
//!
//! 所有持久化写入必须经过 [`write_atomic`]，避免进程中断留下半截文件。

use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::error::CoreError;

static TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// 原子写：先写同目录临时文件，再 rename 到目标路径。
///
/// Windows 上 `rename` 覆盖已存在文件会失败，因此先删除目标文件再 rename。
/// 该窗口极短，且启动会做 reconcile（见技术方案 03 §8.8）。
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), CoreError> {
    let dir = path
        .parent()
        .ok_or_else(|| CoreError::Io(format!("目标路径没有父目录: {}", path.display())))?;
    fs::create_dir_all(dir)
        .map_err(|error| CoreError::Io(format!("创建目录 {} 失败: {error}", dir.display())))?;

    let file_name = path
        .file_name()
        .ok_or_else(|| CoreError::Io(format!("目标路径没有文件名: {}", path.display())))?
        .to_string_lossy();
    let seq = TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let tmp = dir.join(format!(".{file_name}.{}.{seq}.tmp", std::process::id()));

    fs::write(&tmp, bytes)
        .map_err(|error| CoreError::Io(format!("写入临时文件 {} 失败: {error}", tmp.display())))?;

    if path.exists() {
        if let Err(error) = fs::remove_file(path) {
            let _ = fs::remove_file(&tmp);
            return Err(CoreError::Io(format!(
                "替换目标文件 {} 失败: {error}",
                path.display()
            )));
        }
    }

    fs::rename(&tmp, path).map_err(|error| {
        let _ = fs::remove_file(&tmp);
        CoreError::Io(format!("写入 {} 失败: {error}", path.display()))
    })
}
