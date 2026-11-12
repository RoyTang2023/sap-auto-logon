//! 进程检测：用 Toolhelp 快照遍历，几毫秒就能判断进程是否在跑。
//!
//! 之前用 `tasklist` 子进程（100-300ms，还要担心控制台窗口），换成系统快照。

/// 判断指定可执行文件（含 `.exe`）是否正在运行。
#[cfg(target_os = "windows")]
pub fn is_running(exe_name: &str) -> bool {
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };

    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot == INVALID_HANDLE_VALUE {
        return false;
    }

    let mut entry: PROCESSENTRY32W = unsafe { std::mem::zeroed() };
    entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

    let mut has_entry = unsafe { Process32FirstW(snapshot, &mut entry) } != 0;
    while has_entry {
        let length = entry
            .szExeFile
            .iter()
            .position(|&unit| unit == 0)
            .unwrap_or(entry.szExeFile.len());
        let name = String::from_utf16_lossy(&entry.szExeFile[..length]);
        if name.eq_ignore_ascii_case(exe_name) {
            unsafe {
                CloseHandle(snapshot);
            }
            return true;
        }
        has_entry = unsafe { Process32NextW(snapshot, &mut entry) } != 0;
    }

    unsafe {
        CloseHandle(snapshot);
    }
    false
}

#[cfg(not(target_os = "windows"))]
pub fn is_running(_exe_name: &str) -> bool {
    false
}
