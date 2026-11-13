//! 自动更新：读 GitHub Release、下载安装包、交给 NSIS 安装器覆盖。
//!
//! 更新的是**安装包**而不是运行中的 exe：SAP Auto Logon 是 per-user 安装，
//! 安装目录、开始菜单快捷方式和卸载记录都归安装器管，绕过它直接换文件，
//! 卸载项与文件就会对不上。所以这里只负责"找到、下载、交给安装器"。

use std::io::Write;
use std::path::{Path, PathBuf};

use crate::core::paths;

const REPO: &str = "revolveyao/sap-auto-logon";
const USER_AGENT: &str = "sap-auto-logon";

/// 发现的新版本。
#[derive(Clone)]
pub struct Release {
    pub version: String,
    pub notes: String,
    setup_url: String,
}

/// 查最新 Release；没有更新的版本时返回 `None`。
pub fn check() -> anyhow::Result<Option<Release>> {
    let text = get(&format!(
        "https://api.github.com/repos/{REPO}/releases/latest"
    ))?;
    let json: serde_json::Value = serde_json::from_str(&text)?;

    let tag = json["tag_name"].as_str().unwrap_or_default();
    let version = tag.trim_start_matches(['v', 'V']).to_string();
    if version.is_empty() || !is_newer(&version, env!("CARGO_PKG_VERSION")) {
        return Ok(None);
    }

    let asset = json["assets"]
        .as_array()
        .and_then(|assets| {
            assets.iter().find(|asset| {
                asset["name"]
                    .as_str()
                    .is_some_and(|name| name.ends_with("-setup.exe"))
            })
        })
        .ok_or_else(|| anyhow::anyhow!("发布里没有安装包"))?;
    let setup_url = asset["browser_download_url"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    if setup_url.is_empty() {
        return Err(anyhow::anyhow!("发布里没有安装包地址"));
    }

    Ok(Some(Release {
        version,
        notes: json["body"].as_str().unwrap_or_default().trim().to_string(),
        setup_url,
    }))
}

/// 把安装包下到应用数据目录，返回本地路径。
pub fn download(release: &Release) -> anyhow::Result<PathBuf> {
    let dir = paths::local_dir().join("updates");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("SAP-Auto-Logon-{}-setup.exe", release.version));

    let response = ureq::get(&release.setup_url)
        .set("User-Agent", USER_AGENT)
        .call()?;
    let mut reader = response.into_reader();
    let mut file = std::fs::File::create(&path)?;
    std::io::copy(&mut reader, &mut file)?;
    file.flush()?;
    if file.metadata()?.len() < 1024 * 1024 {
        return Err(anyhow::anyhow!("下载的安装包不完整"));
    }
    Ok(path)
}

/// 拉起安装器。调用方必须紧接着退出：安装脚本会先结束旧进程再覆盖文件。
pub fn run_installer(path: &Path) -> anyhow::Result<()> {
    // /S = 静默安装：脚本里已经 taskkill 旧进程，装完自动拉起新版本。
    std::process::Command::new(path).arg("/S").spawn()?;
    Ok(())
}

fn get(url: &str) -> anyhow::Result<String> {
    let response = ureq::get(url)
        .set("User-Agent", USER_AGENT)
        .set("Accept", "application/vnd.github+json")
        .call()?;
    Ok(response.into_string()?)
}

/// 按点分数字逐段比较：`0.2.10` 比 `0.2.9` 新，字符串比较会把顺序判反。
fn is_newer(candidate: &str, current: &str) -> bool {
    fn parts(value: &str) -> Vec<u32> {
        value
            .split('.')
            .map(|part| part.trim().parse().unwrap_or(0))
            .collect()
    }
    let mut left = parts(candidate);
    let mut right = parts(current);
    let len = left.len().max(right.len());
    left.resize(len, 0);
    right.resize(len, 0);
    left > right
}

