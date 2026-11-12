//! 定位 SAP Landscape 文件。
//!
//! 路径不写死：SAP GUI 自己在注册表里记着「最后使用的 Landscape」和
//! 「各进程正在用的 Landscape」。用户的配置换了位置，工具跟着走。

use std::path::PathBuf;

#[cfg(target_os = "windows")]
pub fn landscape_path() -> Option<PathBuf> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let mut candidates: Vec<PathBuf> = Vec::new();

    // 1) 最近一次使用的文件。SAP Logon 退出后它仍然在这里。
    if let Ok(key) = hkcu.open_subkey(r"Software\SAP\SAPLogon\LandscapeFilesLastUsed") {
        if let Ok(value) = key.get_value::<String, _>("LandscapeFile") {
            candidates.push(PathBuf::from(value.trim()));
        }
    }

    // 2) 正在使用的文件。多个进程时取 pid 最大的那个。
    if let Ok(key) = hkcu.open_subkey(r"Software\SAP\SAPLogon") {
        let mut in_use: Vec<(u32, PathBuf)> = Vec::new();
        for name in key.enum_keys().flatten() {
            let Some(pid) = name
                .strip_prefix("LandscapeFilesInUse_")
                .and_then(|pid| pid.parse::<u32>().ok())
            else {
                continue;
            };
            let Ok(subkey) = key.open_subkey(&name) else {
                continue;
            };
            let Ok(value) = subkey.get_value::<String, _>("LandscapeFile") else {
                continue;
            };
            in_use.push((pid, PathBuf::from(value.trim())));
        }
        in_use.sort_by_key(|(pid, _)| std::cmp::Reverse(*pid));
        candidates.extend(in_use.into_iter().map(|(_, path)| path));
    }

    // 3) SAP GUI 的默认位置。
    if let Some(data_dir) = dirs::data_dir() {
        candidates.push(data_dir.join("SAP").join("Common").join("SAPUILandscape.xml"));
    }

    candidates.into_iter().find(|path| path.is_file())
}

#[cfg(not(target_os = "windows"))]
pub fn landscape_path() -> Option<PathBuf> {
    let path = dirs::data_dir()?.join("SAP").join("Common").join("SAPUILandscape.xml");
    path.is_file().then_some(path)
}
