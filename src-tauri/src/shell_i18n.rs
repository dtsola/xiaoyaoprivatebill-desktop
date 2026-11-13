//! 托盘菜单与窗口标题的中英文文案。

/// 桌面壳层（托盘 + 窗口）用到的文案。
pub struct ShellLabels {
    /// 应用短名，用于托盘 tooltip。
    pub app_name: &'static str,
    /// 主窗口标题。
    pub window_title: &'static str,
    /// 托盘「显示主窗口」。
    pub show: &'static str,
    /// 托盘「打开数据目录」。
    pub data_dir: &'static str,
    /// MCP 已开启时的菜单文案（点击后关闭）。
    pub mcp_on: &'static str,
    /// MCP 已关闭时的菜单文案（点击后开启）。
    pub mcp_off: &'static str,
    /// 托盘「退出」。
    pub quit: &'static str,
}

/// 把任意语言标识归一为 `zh-CN` / `en-US`。
///
/// 以 `en` 开头的都视为英文，其余一律回退中文，避免托盘出现空文案。
pub fn normalize_locale(locale: &str) -> &'static str {
    if locale.to_ascii_lowercase().starts_with("en") {
        "en-US"
    } else {
        "zh-CN"
    }
}

/// 按语言返回托盘与窗口文案。
pub fn shell_labels(locale: &str) -> ShellLabels {
    match normalize_locale(locale) {
        "en-US" => ShellLabels {
            app_name: "Xiaoyao Bill",
            window_title: "Xiaoyao Bill Desktop",
            show: "Show Window",
            data_dir: "Open Data Folder",
            mcp_on: "Disable MCP Server",
            mcp_off: "Enable MCP Server",
            quit: "Quit",
        },
        _ => ShellLabels {
            app_name: "小遥账单桌面版",
            window_title: "小遥账单桌面版",
            show: "显示主窗口",
            data_dir: "打开数据目录",
            mcp_on: "关闭 MCP 服务",
            mcp_off: "开启 MCP 服务",
            quit: "退出",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_unknown_locales() {
        assert_eq!(normalize_locale("en-GB"), "en-US");
        assert_eq!(normalize_locale("EN_us"), "en-US");
        assert_eq!(normalize_locale("zh-TW"), "zh-CN");
        assert_eq!(normalize_locale("ja-JP"), "zh-CN");
        assert_eq!(normalize_locale(""), "zh-CN");
    }

    #[test]
    fn english_labels_are_translated() {
        let labels = shell_labels("en-US");
        assert_eq!(labels.show, "Show Window");
        assert_eq!(labels.quit, "Quit");
        assert_eq!(labels.mcp_on, "Disable MCP Server");
        assert_eq!(labels.mcp_off, "Enable MCP Server");
        assert_eq!(labels.window_title, "Xiaoyao Bill Desktop");
    }

    #[test]
    fn chinese_labels_by_default() {
        let labels = shell_labels("zh-CN");
        assert_eq!(labels.show, "显示主窗口");
        assert_eq!(labels.quit, "退出");
        assert_eq!(labels.window_title, "小遥账单桌面版");
    }
}
