# Release 打包：先 cargo build --release，再调 NSIS 生成安装包。
#
# 用法：.\scripts\package.ps1
# 前置：安装 NSIS（winget install NSIS.NSIS）；安装器默认不进 PATH，
#       本脚本会去常见位置找 makensis.exe。

$ErrorActionPreference = 'Stop'

$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

$Version = (Select-String -Path 'Cargo.toml' -Pattern '^version = "(.*)"').Matches[0].Groups[1].Value
Write-Host ">>> packaging version $Version"

cargo build --release
if ($LASTEXITCODE -ne 0) { throw "cargo build --release failed" }

$makensisPath = $null
$command = Get-Command makensis -ErrorAction SilentlyContinue
if ($command) {
    $makensisPath = $command.Source
} else {
    foreach ($candidate in @(
        "${env:ProgramFiles(x86)}\NSIS\makensis.exe",
        "$env:ProgramFiles\NSIS\makensis.exe",
        "$env:LOCALAPPDATA\Programs\NSIS\makensis.exe"
    )) {
        if (Test-Path -LiteralPath $candidate) {
            $makensisPath = $candidate
            break
        }
    }
}

if (-not $makensisPath) {
    throw "makensis not found. Install NSIS (winget install NSIS.NSIS) or run: makensis /DAPP_VERSION=$Version installer\sap-auto-logon.nsi"
}

Write-Host ">>> using $makensisPath"
New-Item -ItemType Directory -Force -Path 'dist' | Out-Null
& $makensisPath "/DAPP_VERSION=$Version" "installer\sap-auto-logon.nsi"
if ($LASTEXITCODE -ne 0) { throw "makensis failed with exit code $LASTEXITCODE" }

Write-Host ">>> done: dist\SAP-Auto-Logon-$Version-setup.exe"
