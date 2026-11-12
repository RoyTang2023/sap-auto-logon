//! 主窗口显隐：关闭按钮收进托盘（隐藏），托盘点击时再显示。
//!
//! gpui 没暴露窗口句柄的稳定入口，这里用 `EnumWindows` 按进程 ID 找自己的顶层窗口，
//! 找到后缓存住——隐藏期间窗口句柄仍然有效，重新枚举反而容易拿到别的窗口。

#[cfg(target_os = "windows")]
static CACHED: std::sync::Mutex<Option<isize>> = std::sync::Mutex::new(None);

#[cfg(target_os = "windows")]
fn find_window() -> Option<windows_sys::Win32::Foundation::HWND> {
    use windows_sys::Win32::Foundation::{BOOL, HWND, LPARAM};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetClassNameW, GetWindowTextLengthW, GetWindowThreadProcessId,
        IsWindowVisible,
    };

    // 只按进程号取第一个会抓到启动早期建的隐藏窗口（托盘图标也有自己的消息窗口），
    // 那样 `SW_HIDE` 就打在错误的窗口上，表现是「点关闭没反应」。按类名打分挑主窗口。
    unsafe extern "system" fn callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let target = &mut *(lparam as *mut (u32, HWND, i32));
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid != target.0 {
            return 1;
        }

        let mut class = [0u16; 128];
        let length =
            GetClassNameW(hwnd, class.as_mut_ptr(), class.len() as i32).max(0) as usize;
        let class = String::from_utf16_lossy(&class[..length]);
        let visible = IsWindowVisible(hwnd) != 0;
        let titled = GetWindowTextLengthW(hwnd) > 0;

        let score = if class == "Zed::Window" {
            3
        } else if visible && titled {
            2
        } else if visible {
            1
        } else {
            0
        };
        if score > target.2 {
            target.1 = hwnd;
            target.2 = score;
        }
        if score == 3 {
            return 0; // 主窗口找到了，不再枚举
        }
        1
    }

    let mut target: (u32, HWND, i32) = (std::process::id(), std::ptr::null_mut(), -1);
    unsafe { EnumWindows(Some(callback), &mut target as *mut _ as LPARAM) };
    if target.1.is_null() {
        None
    } else {
        Some(target.1)
    }
}

#[cfg(target_os = "windows")]
fn main_window() -> Option<windows_sys::Win32::Foundation::HWND> {
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::UI::WindowsAndMessaging::IsWindow;

    if let Ok(guard) = CACHED.lock() {
        if let Some(raw) = *guard {
            let hwnd = raw as HWND;
            if unsafe { IsWindow(hwnd) } != 0 {
                return Some(hwnd);
            }
        }
    }

    let found = find_window();
    if let Some(hwnd) = found {
        if let Ok(mut guard) = CACHED.lock() {
            *guard = Some(hwnd as isize);
        }
    }
    found
}

#[cfg(target_os = "windows")]
pub fn hide() {
    use windows_sys::Win32::UI::WindowsAndMessaging::{IsWindowVisible, ShowWindow, SW_HIDE};
    match main_window() {
        Some(hwnd) => {
            let visible = unsafe { IsWindowVisible(hwnd) } != 0;
            log::info!("tray hide: hwnd={hwnd:p} visible={visible}");
            unsafe {
                ShowWindow(hwnd, SW_HIDE);
            }
        }
        None => log::warn!("tray hide: no window found"),
    }
}

#[cfg(target_os = "windows")]
pub fn show() {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        IsWindowVisible, SetForegroundWindow, ShowWindow, SW_SHOW,
    };
    match main_window() {
        Some(hwnd) => {
            let visible = unsafe { IsWindowVisible(hwnd) } != 0;
            log::info!("tray show: hwnd={hwnd:p} visible={visible}");
            unsafe {
                ShowWindow(hwnd, SW_SHOW);
                SetForegroundWindow(hwnd);
            }
        }
        None => log::warn!("tray show: no window found"),
    }
}

#[cfg(not(target_os = "windows"))]
pub fn hide() {}

#[cfg(not(target_os = "windows"))]
pub fn show() {}
