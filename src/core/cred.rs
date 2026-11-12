//! Windows 凭据管理器：密码交给系统保管，不落盘、不进日志。
//!
//! 直接调 `CredWriteW` / `CredReadW` / `CredDeleteW`，target 用
//! `SapAutoLogon/<快捷方式 uuid>`——uuid 唯一且改名不影响。

#[cfg(target_os = "windows")]
mod imp {
    use std::ffi::c_void;
    use std::ptr;

    use windows_sys::Win32::Foundation::FILETIME;
    use windows_sys::Win32::Security::Credentials::{
        CredFree, CredReadW, CredWriteW, CREDENTIALW, CRED_PERSIST_LOCAL_MACHINE,
        CRED_TYPE_GENERIC,
    };

    fn wide(text: &str) -> Vec<u16> {
        text.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn from_wide(pointer: *const u16) -> String {
        if pointer.is_null() {
            return String::new();
        }
        let mut length = 0usize;
        unsafe {
            while *pointer.add(length) != 0 {
                length += 1;
            }
            String::from_utf16_lossy(std::slice::from_raw_parts(pointer, length))
        }
    }

    pub fn save(target: &str, user: &str, password: &str) -> bool {
        let mut target_wide = wide(target);
        let mut user_wide = wide(user);
        let mut blob: Vec<u8> = password
            .encode_utf16()
            .flat_map(|unit| unit.to_le_bytes())
            .collect();

        let credential = CREDENTIALW {
            Flags: 0,
            Type: CRED_TYPE_GENERIC,
            TargetName: target_wide.as_mut_ptr(),
            Comment: ptr::null_mut(),
            LastWritten: FILETIME {
                dwLowDateTime: 0,
                dwHighDateTime: 0,
            },
            CredentialBlobSize: blob.len() as u32,
            CredentialBlob: blob.as_mut_ptr(),
            Persist: CRED_PERSIST_LOCAL_MACHINE,
            AttributeCount: 0,
            Attributes: ptr::null_mut(),
            TargetAlias: ptr::null_mut(),
            UserName: user_wide.as_mut_ptr(),
        };

        unsafe { CredWriteW(&credential, 0) != 0 }
    }

    pub fn load(target: &str) -> Option<(String, String)> {
        let target_wide = wide(target);
        let mut pointer: *mut CREDENTIALW = ptr::null_mut();
        if unsafe { CredReadW(target_wide.as_ptr(), CRED_TYPE_GENERIC, 0, &mut pointer) } == 0 {
            return None;
        }
        if pointer.is_null() {
            return None;
        }

        let credential = unsafe { &*pointer };
        let user = from_wide(credential.UserName);
        let bytes = unsafe {
            std::slice::from_raw_parts(
                credential.CredentialBlob,
                credential.CredentialBlobSize as usize,
            )
        };
        let units: Vec<u16> = bytes
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();
        let password = String::from_utf16_lossy(&units);
        unsafe { CredFree(pointer as *const c_void) };
        Some((user, password))
    }
}

#[cfg(not(target_os = "windows"))]
mod imp {
    pub fn save(_target: &str, _user: &str, _password: &str) -> bool {
        false
    }

    pub fn load(_target: &str) -> Option<(String, String)> {
        None
    }

    pub fn delete(_target: &str) -> bool {
        false
    }
}

#[allow(unused_imports)]
pub use imp::{load, save};

pub fn target_for(service_uuid: &str) -> String {
    format!("SapAutoLogon/{service_uuid}")
}
