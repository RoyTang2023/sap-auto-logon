//! 构建脚本：把图标与版本信息嵌进 exe。

fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    // i18n 文案在编译期被 rust-i18n 读入，改 yml 必须重新编译。
    println!("cargo:rerun-if-changed=locales");

    #[cfg(target_os = "windows")]
    {
        let version = std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".into());
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        // 资源字符串走 Windows 的 UTF-16 表，但 rc.exe 对非 ASCII 源文本的编码
        // 依赖系统代码页；这里用 ASCII 产品名，界面里再显示中文名。
        res.set("FileDescription", "SAP Auto Logon");
        res.set("ProductName", "SAP Auto Logon");
        res.set("OriginalFilename", "sap-auto-logon.exe");
        res.set("FileVersion", &version);
        res.set("ProductVersion", &version);
        res.compile().expect("failed to compile Windows resources");
    }
}
