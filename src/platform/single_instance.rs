//! 单实例：同名互斥体已存在时说明另一个实例在跑。
//!
//! 互斥体句柄刻意不关闭：进程退出时内核自动释放，中途关闭会让第二个实例挤进来。

#[cfg(target_os = "windows")]
mod imp {
    use std::ffi::c_void;

    use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS};
    use windows_sys::Win32::System::Threading::CreateMutexW;

    pub struct InstanceGuard(#[allow(dead_code)] *mut c_void);

    impl Drop for InstanceGuard {
        fn drop(&mut self) {
            // 句柄永不关闭：见模块注释。
        }
    }

    fn wide(text: &str) -> Vec<u16> {
        text.encode_utf16().chain(std::iter::once(0)).collect()
    }

    pub fn acquire() -> Option<InstanceGuard> {
        let name = wide(r"Local\SapAutoLogon.SingleInstance");
        let handle = unsafe { CreateMutexW(std::ptr::null(), 0, name.as_ptr()) };
        if handle.is_null() {
            return None;
        }
        if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
            unsafe {
                CloseHandle(handle);
            }
            return None;
        }
        Some(InstanceGuard(handle))
    }
}

#[cfg(not(target_os = "windows"))]
mod imp {
    pub struct InstanceGuard;

    pub fn acquire() -> Option<InstanceGuard> {
        Some(InstanceGuard)
    }
}

#[allow(unused_imports)]
pub use imp::{acquire, InstanceGuard};
