//! 文件日志与 panic 钩子。
//!
//! 日志开关与级别来自设置，所以这里对配置做一次窄读：日志器本身要在
//! 「配置损坏」这件事上留一条记录，不能等完整配置加载。

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

use log::{LevelFilter, Log, Metadata, Record};

use crate::core::paths;
use crate::core::settings::AppSettings;

struct FileLogger {
    file: Mutex<std::fs::File>,
}

impl Log for FileLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if let Ok(mut file) = self.file.lock() {
            let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            let _ = writeln!(file, "[{now}] {:<5} {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

pub fn init() {
    let settings = AppSettings::load();
    let general = &settings.general;
    if !general.logging {
        return;
    }

    let dir = paths::log_dir(&general.log_dir);
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    let path = dir.join("sap-auto-logon.log");
    let Ok(file) = OpenOptions::new().create(true).append(true).open(&path) else {
        return;
    };

    let logger = FileLogger {
        file: Mutex::new(file),
    };
    if log::set_boxed_logger(Box::new(logger)).is_ok() {
        log::set_max_level(level_from(&general.log_level));
    }

    install_panic_hook(dir);
}

fn level_from(value: &str) -> LevelFilter {
    match value.trim().to_ascii_lowercase().as_str() {
        "trace" => LevelFilter::Trace,
        "debug" => LevelFilter::Debug,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Info,
    }
}

/// 无控制台的应用里 panic 没人看得见，所以无条件写一份到日志文件。
fn install_panic_hook(dir: PathBuf) {
    std::panic::set_hook(Box::new(move |info| {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let mut text = format!("[{now}] PANIC {info}\n");
        if let Some(location) = info.location() {
            text = format!("[{now}] PANIC at {}:{}: {info}\n", location.file(), location.line());
        }
        let path = dir.join("sap-auto-logon.log");
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
            let _ = file.write_all(text.as_bytes());
        }
    }));
}
