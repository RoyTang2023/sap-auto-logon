//! 开机自启：只在 `HKCU\...\Run` 里维护一条，不碰系统级设置。

#[cfg(target_os = "windows")]
pub fn apply(enabled: bool) -> anyhow::Result<()> {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_SET_VALUE};
    use winreg::RegKey;

    let key = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(r"Software\Microsoft\Windows\CurrentVersion\Run", KEY_SET_VALUE)?;

    if enabled {
        let exe = std::env::current_exe()?;
        key.set_value("SapAutoLogon", &exe.display().to_string())?;
    } else {
        let _ = key.delete_value("SapAutoLogon");
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn apply(_enabled: bool) -> anyhow::Result<()> {
    Ok(())
}
