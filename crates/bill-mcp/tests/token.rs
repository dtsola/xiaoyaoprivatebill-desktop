//! P4-1 契约测试：MCP Token 生成、持久化与轮换。

use bill_core::DataPaths;
use bill_mcp::token::{ensure_token, read_token, rotate_token};

fn setup() -> (tempfile::TempDir, DataPaths) {
    let dir = tempfile::tempdir().expect("临时目录");
    let paths = DataPaths::new(dir.path().to_path_buf());
    paths.ensure_layout().expect("创建布局");
    (dir, paths)
}

#[test]
fn ensure_token_creates_64_hex_chars() {
    let (_dir, paths) = setup();
    let token = ensure_token(&paths).expect("生成 Token");

    assert_eq!(token.len(), 64, "32 字节随机数应为 64 位十六进制");
    assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
    assert_eq!(read_token(&paths).as_deref(), Some(token.as_str()));
}

#[test]
fn ensure_token_is_stable_across_calls() {
    let (_dir, paths) = setup();
    let first = ensure_token(&paths).expect("首次");
    let second = ensure_token(&paths).expect("再次");

    assert_eq!(first, second, "已有 Token 不得被覆盖");
}

#[test]
fn rotate_token_replaces_value() {
    let (_dir, paths) = setup();
    let first = ensure_token(&paths).expect("首次");
    let rotated = rotate_token(&paths).expect("轮换");

    assert_ne!(first, rotated);
    assert_eq!(read_token(&paths).as_deref(), Some(rotated.as_str()));
}

#[test]
fn read_token_returns_none_when_missing_or_blank() {
    let (_dir, paths) = setup();
    assert_eq!(read_token(&paths), None);

    std::fs::write(paths.mcp_token_file(), "  \n").expect("写入空白");
    assert_eq!(read_token(&paths), None);
}
