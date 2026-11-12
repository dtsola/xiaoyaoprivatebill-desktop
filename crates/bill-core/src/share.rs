//! 分享海报：产品官网二维码与 PNG 落盘。
//!
//! 海报图片由前端 Canvas 绘制，本模块只负责两件纯后端的事：
//! 1. 用产品官网地址生成离线二维码（不访问网络）；
//! 2. 校验并保存用户导出的 PNG 文件。

use std::path::{Component, Path, PathBuf};

use base64::Engine as _;
use qrcode::render::svg;
use qrcode::{EcLevel, QrCode};

use crate::error::CoreError;

/// 产品官网地址，海报二维码与文案共用这一处常量。
pub const PRODUCT_SITE_URL: &str = "https://www.xiaoyaosai.com/xiaoyaoprivatebill-desktop/";

/// 海报 PNG 上限：20MB，足够 1080×1440 无损图。
pub const MAX_POSTER_BYTES: usize = 20 * 1024 * 1024;

/// 生成产品官网二维码的 SVG 字符串。
///
/// `pixel_size` 为期望的最小边长（像素），调用方通常传 240 或 360。
///
/// # Errors
/// 当 `pixel_size` 为 0 或二维码编码失败时返回错误。
pub fn qr_svg(pixel_size: u32) -> Result<String, CoreError> {
    if pixel_size == 0 {
        return Err(CoreError::Share("二维码尺寸必须大于 0".into()));
    }
    let code = QrCode::with_error_correction_level(PRODUCT_SITE_URL, EcLevel::M)
        .map_err(|error| CoreError::Share(format!("二维码编码失败: {error}")))?;
    Ok(code
        .render::<svg::Color>()
        .min_dimensions(pixel_size, pixel_size)
        .quiet_zone(true)
        .dark_color(svg::Color("#0B1B33"))
        .light_color(svg::Color("#FFFFFF"))
        .build())
}

/// 校验用户选择的保存路径。
///
/// 规则：非空、不含 NUL、不含 `..`、扩展名为 `.png`、必须有文件名。
///
/// # Errors
/// 路径不满足上述任一规则时返回错误。
pub fn validate_poster_path(path: &str) -> Result<PathBuf, CoreError> {
    if path.trim().is_empty() {
        return Err(CoreError::Share("保存路径不能为空".into()));
    }
    if path.contains('\0') {
        return Err(CoreError::Share("保存路径包含非法字符".into()));
    }
    let target = Path::new(path);
    if target
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(CoreError::Share("保存路径不能包含 ..".into()));
    }
    if target.file_name().is_none() {
        return Err(CoreError::Share("保存路径缺少文件名".into()));
    }
    let is_png = target
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("png"));
    if !is_png {
        return Err(CoreError::Share("海报只支持保存为 PNG".into()));
    }
    Ok(target.to_path_buf())
}

/// 解码前端传来的 PNG（支持 `data:image/png;base64,` 前缀）。
///
/// # Errors
/// Base64 非法、不是 PNG、或超过 [`MAX_POSTER_BYTES`] 时返回错误。
pub fn decode_poster_png(input: &str) -> Result<Vec<u8>, CoreError> {
    let payload = input
        .trim()
        .strip_prefix("data:image/png;base64,")
        .unwrap_or(input.trim());
    if payload.starts_with("data:") {
        return Err(CoreError::Share("海报图片格式必须是 PNG".into()));
    }
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(payload)
        .map_err(|_| CoreError::Share("海报图片数据无效".into()))?;
    validate_png_bytes(&bytes)?;
    Ok(bytes)
}

/// 校验 PNG 字节：魔术头 + 体积上限。
///
/// # Errors
/// 不是 PNG 或超过体积上限时返回错误。
pub fn validate_png_bytes(bytes: &[u8]) -> Result<(), CoreError> {
    const PNG_MAGIC: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
    if !bytes.starts_with(PNG_MAGIC) {
        return Err(CoreError::Share("不是有效的 PNG 图片".into()));
    }
    if bytes.len() > MAX_POSTER_BYTES {
        return Err(CoreError::Share(format!(
            "海报图片超过 {}MB 上限",
            MAX_POSTER_BYTES / 1024 / 1024
        )));
    }
    Ok(())
}

/// 保存海报到用户选择的路径。
///
/// # Errors
/// 路径非法、PNG 非法或写入失败时返回错误。
pub fn save_poster(path: &str, png_base64: &str) -> Result<PathBuf, CoreError> {
    let target = validate_poster_path(path)?;
    let bytes = decode_poster_png(png_base64)?;
    crate::fsutil::write_atomic(&target, &bytes)?;
    Ok(target)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 1×1 透明 PNG。
    const PNG_1PX: &str = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==";

    #[test]
    fn qr_svg_has_requested_size() {
        let svg = qr_svg(240).expect("生成二维码失败");
        assert!(svg.contains("<svg"), "应输出 SVG");
        // 渲染器会把边长向上取整到模块的整数倍，因此断言不小于请求值。
        let width = svg
            .split("width=\"")
            .nth(1)
            .and_then(|rest| rest.split('"').next())
            .and_then(|value| value.parse::<u32>().ok())
            .expect("SVG 应包含 width");
        assert!(width >= 240, "二维码宽度 {width} 小于请求值");
        assert!(svg.contains("viewBox="), "SVG 应包含 viewBox");
    }

    #[test]
    fn qr_svg_is_deterministic() {
        assert_eq!(qr_svg(240).unwrap(), qr_svg(240).unwrap());
    }

    #[test]
    fn qr_svg_rejects_zero_size() {
        assert!(qr_svg(0).is_err());
    }

    #[test]
    fn validate_accepts_png_paths() {
        assert!(validate_poster_path("C:\\Users\\me\\海报.png").is_ok());
        assert!(validate_poster_path("/tmp/poster.PNG").is_ok());
    }

    #[test]
    fn validate_rejects_invalid_paths() {
        assert!(validate_poster_path("").is_err());
        assert!(validate_poster_path("   ").is_err());
        assert!(validate_poster_path("C:\\Users\\me\\poster.jpg").is_err());
        assert!(validate_poster_path("C:\\Users\\me\\..\\poster.png").is_err());
        assert!(validate_poster_path("C:\\Users\\me\\poster\0.png").is_err());
        assert!(validate_poster_path("C:\\Users\\me\\").is_err());
    }

    #[test]
    fn decode_accepts_plain_base64() {
        let bytes = decode_poster_png(PNG_1PX).expect("应解码成功");
        assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
    }

    #[test]
    fn decode_accepts_data_url_prefix() {
        let input = format!("data:image/png;base64,{PNG_1PX}");
        assert!(decode_poster_png(&input).is_ok());
    }

    #[test]
    fn decode_rejects_invalid_base64() {
        assert!(decode_poster_png("不是 base64").is_err());
    }

    #[test]
    fn decode_rejects_non_png() {
        let jpeg = base64::engine::general_purpose::STANDARD.encode(b"hello world");
        assert!(decode_poster_png(&jpeg).is_err());
    }

    #[test]
    fn validate_png_rejects_oversized() {
        assert!(validate_png_bytes(&[0u8; MAX_POSTER_BYTES + 1]).is_err());
    }

    #[test]
    fn save_poster_writes_png() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("share.png");
        let saved = save_poster(target.to_str().unwrap(), PNG_1PX).expect("保存失败");
        assert_eq!(saved, target);
        let bytes = std::fs::read(&target).unwrap();
        assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
    }

    #[test]
    fn product_site_url_matches_readme() {
        assert_eq!(
            PRODUCT_SITE_URL,
            "https://www.xiaoyaosai.com/xiaoyaoprivatebill-desktop/"
        );
    }

    #[test]
    fn path_with_parent_dir_component_is_rejected() {
        let path = Path::new("C:\\Users\\me\\..\\poster.png");
        assert!(path
            .components()
            .any(|component| matches!(component, Component::ParentDir)));
    }
}
