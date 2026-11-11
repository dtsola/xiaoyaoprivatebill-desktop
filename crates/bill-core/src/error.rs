//! 核心错误类型。
//!
//! 错误信息直接面向用户，必须为中文且可读。

/// 领域层与本地存储的错误。
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    /// 数据目录不可用（创建失败、无权限等）。
    #[error("数据目录不可用: {0}")]
    DataDir(String),
    /// 文件读写失败。
    #[error("文件读写失败: {0}")]
    Io(String),
    /// 设置文件损坏或无法解析。
    #[error("设置文件解析失败: {0}")]
    Settings(String),
}
