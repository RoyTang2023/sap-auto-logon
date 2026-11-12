# SAP Auto Logon

面向每天要在多个 SAP 系统之间来回切换的用户：把 SAP GUI 里那棵树变成一排卡片，双击即登录。

## 简介

SAP GUI 自带的 SAP Logon 把系统藏在树形目录和右键菜单里，登录一次要点开节点、双击系统、填客户端、输用户和密码，换一套系统再来一遍。这个工具直接读 SAP GUI 自己的 `SAPUILandscape.xml`，把里面的快捷方式铺成卡片墙：最近用过的排最前，双击直接拉起 `sapshcut.exe` 登录，密码存在 Windows 凭据管理器里，不用每次重输。它同时能反向写回 Landscape —— 新建、修改、删除快捷方式，结果立刻在 SAP GUI 里可见，写之前自动备份。

> 只读你本机的 SAP GUI 配置；写回操作前会自动备份，且只动快捷方式（`Reference`），不碰基础系统。

## 下载

到 [Releases](../../releases/latest) 下载 `SAP-Auto-Logon-<版本>-setup.exe`。安装是 per-user 的（装到 `%LOCALAPPDATA%\Programs\SAP Auto Logon`），不需要管理员权限。

也可以直接拿单文件 `sap-auto-logon.exe` 跑，纯绿色。

## 功能

| 功能 | 说明 |
|---|---|
| Landscape 定位 | 读注册表 `LandscapeFilesLastUsed` / `LandscapeFilesInUse_<pid>`，回退 `%APPDATA%\SAP\Common` |
| 解析与缓存 | 缓存 `%LOCALAPPDATA%\SapAutoLogon\cache.json`，按源文件 `mtime + size` + 程序版本失效 |
| 卡片网格 | 按文件夹分组、每行四张、最近登录置顶、可固定、可拖动排序 |
| 登录 | 双击卡片；已存密码直接调 `sapshcut.exe`，没存则弹密码框 |
| 密码存储 | Windows 凭据管理器（`CredWriteW`），目标名 `SapAutoLogon/<uuid>` |
| 状态灯 | 灰=没密码 / 蓝=已存密码 / 绿=上次成功 / 红=上次失败 |
| 右键菜单 | 登录、设置密码、编辑参数、设置别名、固定、上移下移、删除 |
| 新建 | 标题栏 `+`：选文件夹（含「顶层」）与基础系统，名称自动拼，客户端校验三位数字 |
| 写回 | 只写 `Reference`；写前备份到 `%LOCALAPPDATA%\SapAutoLogon\backups`（保留 5 份）；SAP Logon 在运行就拒绝 |
| 托盘 | 关闭收进托盘；菜单打开主窗口 / 设置 / 退出 |
| 外观 | 跟随系统 / 浅色 / 深色；材质 Mica（Win11 22H2+）/ 亚克力 / 不透明 |
| SAP GUI 特性 | UUID 模式：8.10 及以上默认开启，交给 SAP GUI 自己解析系统 |
| 国际化 | `locales/ui.yml`（en / zh-CN），跟随系统语言 |
| 单实例 | `Local\SapAutoLogon.SingleInstance` |

## 使用

1. 打开程序，它会自动找到 SAP GUI 用着的 Landscape。
2. 双击卡片登录。第一次会问密码，勾选记住后写进凭据管理器。
3. 标题栏 `+` 新建快捷方式，结果会写回 `SAPUILandscape.xml`（SAP GUI 里立即可见）。
4. 点 `✕` 收进托盘；退出走托盘菜单。

## 构建

需要 Rust stable（本机验证 1.99）和 Windows。

```powershell
cargo build            # 调试构建
cargo build --release  # 发布构建
```

图标由 `tools/gen-icon.py` 生成（需要 Pillow），`build.rs` 把它和版本信息嵌进 exe。

## 打包安装包

```powershell
.\scripts\package.ps1
```

需要 [NSIS](https://nsis.sourceforge.io/)（脚本会自动去常见位置找 `makensis.exe`），输出 `dist\SAP-Auto-Logon-<版本>-setup.exe`。

## 发布

推一个与 `Cargo.toml` 版本一致的 tag 即可，GitHub Actions 会编译、打包并挂到 Release：

```powershell
git tag v0.2.2
git push origin v0.2.2
```

Workflow：`.github/workflows/release.yml`（windows-latest + `choco install nsis`）。手动触发也能跑，只是不出 Release 只出构建产物。

## 目录

```
src/
├── main.rs              启动、窗口选项、i18n 初始化
├── core/                paths / settings / cache / cred
├── landscape/           locate / parse / model / write
├── platform/            tray / window / single_instance / autostart / system
├── services/            logging / shcut
└── ui/                  root（卡片 + 弹框 + 设置）/ card
locales/ui.yml           中英文案
installer/               NSIS 脚本
scripts/package.ps1      本地打包
tools/gen-icon.py        图标生成
docs/设计文档.md          设计取舍与数据结构
```

## 隐私

- 密码只进 Windows 凭据管理器，不落配置文件、不进日志（`sapshcut` 命令行里的 `-pw=` 打码）。
- 不回传任何数据；Landscape 与缓存都只在本机。

## 许可

[MIT](LICENSE)
