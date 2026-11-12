//! 连接清单缓存。
//!
//! 解析一次 Landscape 要遍历 136 个 Service 与 21 个 Node，冷启动时不算慢，
//! 但每次打开都读 XML 没有必要：用源文件的 `mtime + size` 判断是否需要重解析。

use std::path::Path;
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};

use crate::core::paths;
use crate::landscape::model::Landscape;

/// 缓存结构变化时必须递增，否则旧缓存会被当成新结构读。
/// 2：修复了「Workspaces 排在 Services 之前导致树为空」的解析 bug，旧缓存作废。
pub const SCHEMA: u32 = 2;

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheFile {
    pub schema: u32,
    pub source: String,
    pub source_mtime: u64,
    pub source_size: u64,
    pub parsed_at: String,
    pub landscape: Landscape,
    /// 上次扫描时已保存密码的快捷方式 uuid：启动先用它点亮状态灯，后台再校正。
    #[serde(default)]
    pub password_ready: Vec<String>,
}

pub fn source_stamp(path: &Path) -> Option<(u64, u64)> {
    let meta = std::fs::metadata(path).ok()?;
    let mtime = meta
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_secs();
    Some((mtime, meta.len()))
}

pub fn load_if_fresh(source: &Path) -> Option<Landscape> {
    let (mtime, size) = source_stamp(source)?;
    let text = std::fs::read_to_string(paths::cache_path()).ok()?;
    let cache: CacheFile = serde_json::from_str(&text).ok()?;
    if cache.schema != SCHEMA || cache.source_mtime != mtime || cache.source_size != size {
        return None;
    }
    Some(cache.landscape)
}

pub fn save(source: &Path, landscape: &Landscape) -> anyhow::Result<()> {
    let (mtime, size) = source_stamp(source)
        .ok_or_else(|| anyhow::anyhow!("无法读取源文件时间戳：{}", source.display()))?;
    let cache = CacheFile {
        schema: SCHEMA,
        source: source.display().to_string(),
        source_mtime: mtime,
        source_size: size,
        parsed_at: chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%:z").to_string(),
        landscape: landscape.clone(),
        // 重新解析时保留上一次扫描到的凭据状态，避免启动灯全灭。
        password_ready: read_cache()
            .map(|cache| cache.password_ready)
            .unwrap_or_default(),
    };
    let path = paths::cache_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string(&cache)?;
    std::fs::write(&path, text)?;
    Ok(())
}

/// 读取缓存文件（不校验新鲜度），供解析保存与凭据更新复用。
fn read_cache() -> Option<CacheFile> {
    let text = std::fs::read_to_string(paths::cache_path()).ok()?;
    serde_json::from_str(&text).ok()
}

/// 启动时读上次扫描的凭据状态：缓存与源文件匹配才用。
pub fn load_password_ready(source: &Path) -> Vec<String> {
    let Some((mtime, size)) = source_stamp(source) else {
        return Vec::new();
    };
    let Some(cache) = read_cache() else {
        return Vec::new();
    };
    if cache.schema != SCHEMA || cache.source_mtime != mtime || cache.source_size != size {
        return Vec::new();
    }
    cache.password_ready
}

/// 后台扫描 / 设置密码后回写凭据状态，下次启动即可提前点亮状态灯。
pub fn save_password_ready(source: &Path, ready: Vec<String>) {
    let Some((mtime, size)) = source_stamp(source) else {
        return;
    };
    let Some(mut cache) = read_cache() else {
        return;
    };
    if cache.schema != SCHEMA || cache.source_mtime != mtime || cache.source_size != size {
        return;
    }
    cache.password_ready = ready;
    if let Ok(text) = serde_json::to_string_pretty(&cache) {
        let _ = std::fs::write(paths::cache_path(), text);
    }
}