//! 目录选择：在后台线程跑 Windows 原生对话框，结果经 channel 回传。
//!
//! 不能在 UI 线程同步等待模态对话框——gpui 会在阻塞期间重入 `App` 借用直接
//! panic（日志里是 `RefCell already borrowed`）。所以这里只负责"起线程 + 收结果"，
//! 主线程用定时器轮询 channel。

use std::sync::mpsc::{self, Receiver};

/// 在后台线程弹出目录选择框；返回的 channel 只会收到一条消息。
pub fn pick_folder_async(title: &'static str) -> Receiver<Option<String>> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let result = pick_folder_blocking(title);
        let _ = tx.send(result);
    });
    rx
}

#[cfg(target_os = "windows")]
fn pick_folder_blocking(title: &'static str) -> Option<String> {
    use std::ptr;
    use windows_sys::Win32::System::Com::CoTaskMemFree;
    use windows_sys::Win32::UI::Shell::{
        BROWSEINFOW, SHBrowseForFolderW, SHGetPathFromIDListW, BIF_RETURNONLYFSDIRS,
    };

    let title_wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
    let mut display = [0u16; 260];
    let mut info: BROWSEINFOW = unsafe { std::mem::zeroed() };
    info.hwndOwner = ptr::null_mut();
    info.pszDisplayName = display.as_mut_ptr();
    info.lpszTitle = title_wide.as_ptr();
    info.ulFlags = BIF_RETURNONLYFSDIRS;

    let pidl = unsafe { SHBrowseForFolderW(&info) };
    if pidl.is_null() {
        return None;
    }
    let mut path = [0u16; 260];
    let ok = unsafe { SHGetPathFromIDListW(pidl, path.as_mut_ptr()) };
    unsafe { CoTaskMemFree(pidl as *mut core::ffi::c_void) };
    if ok == 0 {
        return None;
    }
    let len = path.iter().position(|&c| c == 0).unwrap_or(path.len());
    Some(String::from_utf16_lossy(&path[..len]))
}

#[cfg(not(target_os = "windows"))]
fn pick_folder_blocking(_title: &'static str) -> Option<String> {
    None
}
