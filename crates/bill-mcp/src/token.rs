//! MCP Token：生成、持久化与轮换。
//!
//! Token 只写入 `mcp_token` 文件，**不得进入日志、审计或错误信息**。

use bill_core::{write_atomic, DataPaths};

/// Token 随机字节数；十六进制表示后固定 64 字符。
pub const TOKEN_BYTES: usize = 32;

const HEX: [u8; 16] = *b"0123456789abcdef";

/// 读取已有 Token；文件缺失或内容为空时返回 `None`。
pub fn read_token(paths: &DataPaths) -> Option<String> {
    let text = std::fs::read_to_string(paths.mcp_token_file()).ok()?;
    let token = text.trim();
    if token.is_empty() {
        None
    } else {
        Some(token.to_owned())
    }
}

/// 读取 Token；不存在则生成并落盘。
pub fn ensure_token(paths: &DataPaths) -> Result<String, String> {
    if let Some(token) = read_token(paths) {
        return Ok(token);
    }
    generate_and_save(paths)
}

/// 强制轮换：生成新 Token 覆盖旧值。
pub fn rotate_token(paths: &DataPaths) -> Result<String, String> {
    generate_and_save(paths)
}

fn generate_and_save(paths: &DataPaths) -> Result<String, String> {
    let token = random_hex().map_err(|error| format!("生成 MCP Token 失败: {error}"))?;
    write_atomic(&paths.mcp_token_file(), token.as_bytes())
        .map_err(|error| format!("写入 MCP Token 失败: {error}"))?;
    Ok(token)
}

fn random_hex() -> Result<String, getrandom::Error> {
    let mut bytes = [0u8; TOKEN_BYTES];
    getrandom::fill(&mut bytes)?;
    let mut token = String::with_capacity(TOKEN_BYTES * 2);
    for byte in bytes {
        token.push(HEX[(byte >> 4) as usize] as char);
        token.push(HEX[(byte & 0x0f) as usize] as char);
    }
    Ok(token)
}
