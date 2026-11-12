//! 主窗口显隐：关闭按钮收进托盘（隐藏），托盘点击时再显示。
//!
//! gpui 没暴露窗口句柄的稳定入口，这里用 `EnumWindows` 按进程 ID 找自己的顶层窗口。

#[cfg(target_os = "windows")]
fn main_window() -> Option<windows_sys::Win32::Foundation::HWND> {
    use windows_sys::Win32::Foundation::{BOOL, HWND, LPARAM};
    use windows_sys::Win32::UI::WindowsAndMessaging::{EnumWindows, GetWindowThreadProcessId};

    unsafe extern "system" fn callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let target = &mut *(lparam as *mut (u32, HWND));
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == target.0 && target.1.is_null() {
            target.1 = hwnd;
            return 0; // 找到就停
        }
        1
    }

    let mut target: (u32, HWND) = (std::process::id(), std::ptr::null_mut());
    unsafe { EnumWindows(Some(callback), &mut target as *mut _ as LPARAM) };
    if target.1.is_null() {
        None
    } else {
        Some(target.1)
    }
}

#[cfg(target_os = "windows")]
pub fn hide() {
    use windows_sys::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE};
    if let Some(hwnd) = main_window() {
        unsafe {
            ShowWindow(hwnd, SW_HIDE);
        }
    }
}

#[cfg(target_os = "windows")]
pub fn show() {
    use windows_sys::Win32::UI::WindowsAndMessaging::{SetForegroundWindow, ShowWindow, SW_SHOW};
    if let Some(hwnd) = main_window() {
        unsafe {
            ShowWindow(hwnd, SW_SHOW);
            SetForegroundWindow(hwnd);
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn hide() {}

#[cfg(not(target_os = "windows"))]
pub fn show() {}
