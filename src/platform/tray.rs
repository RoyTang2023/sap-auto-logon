//! 系统托盘：图标、菜单与事件轮询。
//! 系统托盘：图标、菜单与事件轮询。
//!
//! `tray-icon` 把事件投递到全局通道，轮询在 UI 线程完成，因此窗口状态仍是单线程的。

use rust_i18n::t;
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder, TrayIconEvent};

/// 托盘菜单触发的动作。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayAction {
    /// 双击图标或点「打开主窗口」
    ShowWindow,
    /// 打开设置
    Settings,
    Quit,
}

pub struct Tray {
    _tray: TrayIcon,
    show_id: tray_icon::menu::MenuId,
    settings_id: tray_icon::menu::MenuId,
    quit_id: tray_icon::menu::MenuId,
    // 菜单项必须一直存活：底层 muda 持有它们的引用，丢弃会变成空白项。
    _version: MenuItem,
    _separator: PredefinedMenuItem,
    _show: MenuItem,
    _settings: MenuItem,
    _quit: MenuItem,
}

impl Tray {
    pub fn create() -> Result<Self, String> {
        let menu = Menu::new();
        let version = MenuItem::new(
            format!("SAP Auto Logon v{}", env!("CARGO_PKG_VERSION")),
            false,
            None,
        );
        let separator = PredefinedMenuItem::separator();
        let show = MenuItem::new(t!("app.tray_show").to_string(), true, None);
        let settings = MenuItem::new(t!("app.tray_settings").to_string(), true, None);
        let quit = MenuItem::new(t!("app.tray_quit").to_string(), true, None);

        menu.append_items(&[&version, &separator, &show, &settings, &quit])
            .map_err(|error| format!("构建托盘菜单失败：{error}"))?;

        let tray = TrayIconBuilder::new()
            .with_icon(load_icon()?)
            .with_menu(Box::new(menu))
            .with_tooltip(t!("app.tray_tooltip").to_string())
            .build()
            .map_err(|error| format!("创建托盘图标失败：{error}"))?;

        Ok(Self {
            _tray: tray,
            show_id: show.id().clone(),
            settings_id: settings.id().clone(),
            quit_id: quit.id().clone(),
            _version: version,
            _separator: separator,
            _show: show,
            _settings: settings,
            _quit: quit,
        })
    }

    /// 排空待处理事件，返回第一个识别出的动作。
    pub fn poll(&self) -> Option<TrayAction> {
        let mut action = None;

        while let Ok(event) = TrayIconEvent::receiver().try_recv() {
            if matches!(event, TrayIconEvent::DoubleClick { .. }) && action.is_none() {
                action = Some(TrayAction::ShowWindow);
            }
        }

        while let Ok(event) = MenuEvent::receiver().try_recv() {
            let recognised = if event.id == self.show_id {
                Some(TrayAction::ShowWindow)
            } else if event.id == self.settings_id {
                Some(TrayAction::Settings)
            } else if event.id == self.quit_id {
                Some(TrayAction::Quit)
            } else {
                None
            };
            if action.is_none() {
                action = recognised;
            }
        }

        action
    }
}

/// 内嵌的 32x32 PNG 解成 RGBA。
fn load_icon() -> Result<Icon, String> {
    let decoded = image::load_from_memory(include_bytes!("../../assets/tray-32.png"))
        .map_err(|error| format!("解码托盘图标失败：{error}"))?;
    let rgba = decoded.to_rgba8();
    let (width, height) = rgba.dimensions();
    Icon::from_rgba(rgba.into_raw(), width, height)
        .map_err(|error| format!("构建托盘图标失败：{error}"))
}

