//! 账单解析器：支付宝 CSV、微信 CSV/XLSX。

pub mod alipay;
mod common;
pub mod error;
pub mod filename;
pub mod wechat;

pub use error::ParseError;
