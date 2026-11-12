//! 路径与数据目录。

use std::path::PathBuf;

const APP_NAME: &str = "SapAutoLogon";

/// 首次启动时确定数据目录，之后的配置、缓存、日志都从它派生。
pub fn init() {
    let _ = config_dir();
}

/// 配置目录（Roaming）：设置文件放这里，跟着用户漫游。
pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(APP_NAME)
}

/// 缓存与日志（Local）：它们跟着这台机器走，不该随漫游配置跨机同步。
pub fn local_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(APP_NAME)
}

pub fn settings_path() -> PathBuf {
    config_dir().join("settings.toml")
}

pub fn cache_path() -> PathBuf {
    local_dir().join("cache.json")
}

pub fn log_dir(configured: &str) -> PathBuf {
    let trimmed = configured.trim();
    if trimmed.is_empty() {
        local_dir().join("logs")
    } else {
        PathBuf::from(trimmed)
    }
}
