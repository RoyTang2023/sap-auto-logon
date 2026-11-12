; SAP Auto Logon 的 NSIS 安装脚本。
;
; 用法（需要先 `cargo build --release`）：
;   makensis /DAPP_VERSION=0.1.0 installer\sap-auto-logon.nsi
;
; 三条刻意的选择：
;   1. per-user 安装到 %LOCALAPPDATA%\Programs\SAP Auto Logon，不弹 UAC。
;   2. 装前先请旧实例退出（--quit 没有实现，所以直接 taskkill）——exe 被占用时
;      后面那行 File 会失败，NSIS 只会弹「中止/重试」。
;   3. 卸载保留 %APPDATA%\SapAutoLogon 的设置；凭据在 Windows 凭据管理器里，
;      安装器不碰。

Unicode true
!include "MUI2.nsh"

!ifndef APP_VERSION
  !error "缺少 APP_VERSION：makensis /DAPP_VERSION=<版本号> installer\sap-auto-logon.nsi"
!endif

!define PRODUCT_NAME "SAP Auto Logon"
!define PRODUCT_EXE "sap-auto-logon.exe"
!define UNINSTALL_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\SapAutoLogon"

Name "${PRODUCT_NAME} ${APP_VERSION}"
OutFile "..\dist\SAP-Auto-Logon-${APP_VERSION}-setup.exe"
InstallDir "$LOCALAPPDATA\Programs\SAP Auto Logon"
InstallDirRegKey HKCU "Software\SapAutoLogon" "InstallDir"
RequestExecutionLevel user
SetCompressor /SOLID lzma

VIProductVersion "${APP_VERSION}.0"
VIAddVersionKey "ProductName" "${PRODUCT_NAME}"
VIAddVersionKey "FileDescription" "${PRODUCT_NAME} installer"
VIAddVersionKey "FileVersion" "${APP_VERSION}"
VIAddVersionKey "LegalCopyright" "MIT"

!define MUI_ICON "..\assets\icon.ico"
!define MUI_UNICON "..\assets\icon.ico"
!define MUI_ABORTWARNING
!define MUI_FINISHPAGE_RUN "$INSTDIR\${PRODUCT_EXE}"
!define MUI_FINISHPAGE_RUN_TEXT "启动 SAP Auto Logon（启动后常驻托盘）"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "SimpChinese"
!insertmacro MUI_LANGUAGE "English"

LangString NAME_Main ${LANG_SIMPCHINESE} "${PRODUCT_NAME} 主程序"
LangString NAME_Main ${LANG_ENGLISH} "${PRODUCT_NAME}"
LangString NAME_Desktop ${LANG_SIMPCHINESE} "桌面快捷方式"
LangString NAME_Desktop ${LANG_ENGLISH} "Desktop shortcut"
LangString DESC_Main ${LANG_SIMPCHINESE} "主程序与托盘图标。"
LangString DESC_Main ${LANG_ENGLISH} "The application and its tray icon."
LangString DESC_Desktop ${LANG_SIMPCHINESE} "在桌面上放一个快捷方式。"
LangString DESC_Desktop ${LANG_ENGLISH} "Put a shortcut on the desktop."

Section "$(NAME_Main)" SecMain
  SectionIn RO

  ; 旧实例占着 exe 时写入会失败，先结束它。
  nsExec::Exec 'taskkill /IM ${PRODUCT_EXE} /F'
  Pop $0
  Sleep 500

  SetOutPath "$INSTDIR"
  File "..\target\release\${PRODUCT_EXE}"
  WriteUninstaller "$INSTDIR\uninstall.exe"

  CreateDirectory "$SMPROGRAMS\${PRODUCT_NAME}"
  CreateShortcut "$SMPROGRAMS\${PRODUCT_NAME}\${PRODUCT_NAME}.lnk" "$INSTDIR\${PRODUCT_EXE}"
  CreateShortcut "$SMPROGRAMS\${PRODUCT_NAME}\卸载 ${PRODUCT_NAME}.lnk" "$INSTDIR\uninstall.exe"

  WriteRegStr HKCU "Software\SapAutoLogon" "InstallDir" "$INSTDIR"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayName" "${PRODUCT_NAME}"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayVersion" "${APP_VERSION}"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayIcon" "$INSTDIR\${PRODUCT_EXE}"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "UninstallString" '"$INSTDIR\uninstall.exe"'
  WriteRegDWORD HKCU "${UNINSTALL_KEY}" "NoModify" 1
  WriteRegDWORD HKCU "${UNINSTALL_KEY}" "NoRepair" 1
SectionEnd

Section /o "$(NAME_Desktop)" SecDesktop
  CreateShortcut "$DESKTOP\${PRODUCT_NAME}.lnk" "$INSTDIR\${PRODUCT_EXE}"
SectionEnd

!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
  !insertmacro MUI_DESCRIPTION_TEXT ${SecMain} $(DESC_Main)
  !insertmacro MUI_DESCRIPTION_TEXT ${SecDesktop} $(DESC_Desktop)
!insertmacro MUI_FUNCTION_DESCRIPTION_END

Section "Uninstall"
  nsExec::Exec 'taskkill /IM ${PRODUCT_EXE} /F'
  Pop $0
  Sleep 500

  Delete "$INSTDIR\${PRODUCT_EXE}"
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"
  Delete "$SMPROGRAMS\${PRODUCT_NAME}\${PRODUCT_NAME}.lnk"
  Delete "$SMPROGRAMS\${PRODUCT_NAME}\卸载 ${PRODUCT_NAME}.lnk"
  RMDir "$SMPROGRAMS\${PRODUCT_NAME}"
  Delete "$DESKTOP\${PRODUCT_NAME}.lnk"

  DeleteRegKey HKCU "${UNINSTALL_KEY}"
  DeleteRegKey HKCU "Software\SapAutoLogon"

  ; 用户数据保留：配置在 %APPDATA%\SapAutoLogon，密码在凭据管理器。
  MessageBox MB_OKCANCEL|MB_ICONINFORMATION \
    "配置保留在：$\r$\n$APPDATA\SapAutoLogon$\r$\n$\r$\n要一起删掉吗？（Windows 凭据里的密码需在「凭据管理器」中删除）" \
    IDOK remove_data IDCANCEL keep_data

  remove_data:
    Delete "$APPDATA\SapAutoLogon\settings.toml"
    Delete "$LOCALAPPDATA\SapAutoLogon\cache.json"
    RMDir /r "$LOCALAPPDATA\SapAutoLogon\logs"
    RMDir "$APPDATA\SapAutoLogon"
    RMDir "$LOCALAPPDATA\SapAutoLogon"
    Goto done
  keep_data:
  done:
SectionEnd
