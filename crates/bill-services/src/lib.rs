//! 业务服务层：数据集加载、导入、统计与图表数据。

pub mod dataset;

pub use dataset::{demo_dataset, load_dataset, ServiceError};
