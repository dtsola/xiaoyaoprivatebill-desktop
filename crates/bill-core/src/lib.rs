//! 小遥账单桌面版核心领域模型。
//!
//! 提供交易模型、数据目录布局、设置读写与原子写工具。

pub mod error;
pub mod fsutil;
pub mod model;
pub mod paths;
pub mod session;
pub mod settings;

pub use error::CoreError;
pub use fsutil::write_atomic;
pub use model::{Dataset, FileEntry, Transaction};
pub use paths::DataPaths;
pub use session::Session;
pub use settings::{DataSettings, DesktopSettings, McpSettings, Settings};
