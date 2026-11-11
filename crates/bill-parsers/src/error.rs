//! 解析错误。

/// 账单解析错误，信息直接面向用户。
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    /// 文件既不是合法 GBK，也不是合法 UTF-8。
    #[error("无法识别文件编码（GBK/UTF-8）")]
    Decode,
    /// 找不到支付宝账单表头行。
    #[error("无法找到支付宝账单表头行")]
    AlipayHeader,
    /// 找不到微信账单表头行。
    #[error("无法找到微信账单表头行")]
    WechatHeader,
    /// 缺少必要列。
    #[error("支付宝账单缺少必要列: {0}")]
    MissingColumn(String),
    /// 金额字段无法解析。
    #[error("金额格式无法识别: {0}")]
    Amount(String),
}
