# SAP Auto Logon

读 SAP GUI 自己的 `SAPUILandscape.xml`，把快捷方式铺成卡片，双击直接登录，不再每次输密码。

- 设计文档：[docs/设计文档.md](docs/设计文档.md)
- 技术栈：Rust + [gpui-kit](https://gpui-kit.com) 0.9.1

## 构建

```powershell
cargo build            # 调试构建
cargo build --release  # 发布构建
```

图标由 `tools/gen-icon.py` 生成（需要 Pillow），`build.rs` 把它嵌进 exe。

## 打包安装包

```powershell
.\scripts\package.ps1
```

需要先安装 [NSIS](https://nsis.sourceforge.io/) 并把 `makensis` 加进 PATH；输出 `dist\SAP-Auto-Logon-<版本>-setup.exe`。安装是 per-user（`%LOCALAPPDATA%\Programs\SAP Auto Logon`），装前会结束旧实例。

## 功能

| 功能 | 说明 |
|---|---|
| Landscape 定位 | 注册表 `LandscapeFilesLastUsed` / `LandscapeFilesInUse_<pid>`，回退 `%APPDATA%` |
| 解析与缓存 | 缓存到 `%LOCALAPPDATA%\SapAutoLogon\cache.json`，`mtime + size` 脏检查 |
| 卡片网格 | 按文件夹分组、最近登录置顶、可固定、可上移下移 |
| 登录 | 双击卡片；有密码直接调 `sapshcut`，没有则弹密码框（存 Windows 凭据管理器） |
| 状态灯 | 灰=没密码 / 蓝=已保存 / 绿=成功 / 红=失败 |
| 右键菜单 | 登录、设置密码、编辑参数、删除（写回 XML）、固定、排序 |
| 新建 | 标题栏 `+`：选文件夹与基础系统、名称自动拼、客户端必填 |
| 写回 | 只做 `Reference`；写前备份 `.bak-时间戳`，SAP Logon 在跑就拒绝 |
| 托盘 | 打开主窗口 / 设置 / 退出，双击图标打开 |
| 设置页 | 主题（跟随系统/浅色/深色）、材质、语言、开机自启、日志、`-uuid` 实验模式 |
| 国际化 | `locales/ui.yml`（en / zh-CN），切换后立即生效 |
| 单实例 | `Local\SapAutoLogon.SingleInstance` |
| 日志 | `%LOCALAPPDATA%\SapAutoLogon\logs\sap-auto-logon.log`，含 panic 记录 |

## 目录

```
src/
├── main.rs              启动、窗口选项、i18n 初始化
├── core/                paths / settings / cache / cred
├── landscape/           locate / parse / model / write
├── platform/            tray / single_instance / autostart
├── services/            logging / shcut
└── ui/                  root（卡片 + 弹框 + 设置）
locales/ui.yml           中英文案
installer/               NSIS 脚本
scripts/package.ps1      发布打包
tools/gen-icon.py        图标生成
```
