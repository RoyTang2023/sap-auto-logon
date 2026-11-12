//! 启动 SAP GUI。
//!
//! 只做一件事：拼 `sapshcut.exe` 的命令行并拉起进程。密码按 sapshcut 的
//! 约束以 `-pw=` 传入，所以日志里必须打码。

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::landscape::model::{Landscape, Service};

/// 找 sapshcut.exe：设置里指定 > 注册表 `SAPsysdir` > 常见安装位置。
pub fn find_sapshcut(configured: &str) -> Option<PathBuf> {
    let configured = configured.trim();
    if !configured.is_empty() {
        let path = PathBuf::from(configured);
        if path.is_file() {
            return Some(path);
        }
    }

    #[cfg(target_os = "windows")]
    {
        use winreg::enums::HKEY_LOCAL_MACHINE;
        use winreg::RegKey;

        for key in [
            r"SOFTWARE\SAP\SAP Shared",
            r"SOFTWARE\WOW6432Node\SAP\SAP Shared",
        ] {
            let Ok(key) = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(key) else {
                continue;
            };
            let Ok(sysdir) = key.get_value::<String, _>("SAPsysdir") else {
                continue;
            };
            let path = PathBuf::from(sysdir).join("sapshcut.exe");
            if path.is_file() {
                return Some(path);
            }
        }
    }

    for candidate in [
        r"C:\Program Files\SAP\FrontEnd\SAPgui\sapshcut.exe",
        r"C:\Program Files (x86)\SAP\FrontEnd\SAPGUI\sapshcut.exe",
    ] {
        let path = PathBuf::from(candidate);
        if path.is_file() {
            return Some(path);
        }
    }

    None
}

/// 一次登录需要的全部参数。密码不参与结构体展示，避免误进日志。
pub struct LoginRequest {
    pub client: String,
    pub user: String,
    pub password: String,
    pub language: String,
    pub use_uuid: bool,
}

/// 构造 sapshcut 参数。`base` 必须是快捷方式指向的基础系统。
pub fn build_args(landscape: &Landscape, base: &Service, request: &LoginRequest) -> Vec<String> {
    let language = if request.language.trim().is_empty() {
        "ZH"
    } else {
        request.language.trim()
    };

    let mut args = vec![
        "-maxgui".to_string(),
        format!("-language={language}"),
        format!("-client={}", request.client.trim()),
        format!("-user={}", request.user.trim()),
        format!("-pw={}", request.password),
        format!("-system={}", base.systemid),
    ];

    if request.use_uuid {
        args.push(format!("-uuid={}", base.uuid));
    } else {
        args.push(format!("-guiparm={}", landscape.guiparm(base)));
    }

    args
}

pub fn launch(exe: &Path, args: &[String]) -> std::io::Result<()> {
    // CREATE_NO_WINDOW：本程序是 windows 子系统，不隐藏的话子进程可能带出控制台窗口。
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt as _;
        Command::new(exe)
            .creation_flags(0x0800_0000)
            .args(args)
            .spawn()
            .map(|_| ())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Command::new(exe).args(args).spawn().map(|_| ())
    }
}

/// 日志里显示的命令行：`-pw=` 打码。
pub fn masked_command(exe: &Path, args: &[String]) -> String {
    let mut masked: Vec<String> = args
        .iter()
        .map(|argument| {
            if argument.starts_with("-pw=") {
                "-pw=****".to_string()
            } else {
                argument.clone()
            }
        })
        .collect();
    masked.insert(0, exe.display().to_string());
    masked.join(" ")
}
