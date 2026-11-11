//! 业务服务层：数据集加载、文件管理、统计与图表数据。

pub mod analysis;
pub mod category;
pub mod dataset;
pub mod files;
pub mod transactions;

pub use dataset::{demo_dataset, load_dataset, ServiceError};
pub use files::{clear_data, delete_file, import_paths, list_files, FileError, MAX_UPLOAD_SIZE};
