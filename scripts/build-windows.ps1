# WeaselVision Windows 构建脚本（x64 + ARM64 / NSIS 安装包）
#
# 前置条件（只需装一次）：
#   1) Visual Studio 生成工具 2022，勾选：
#      - 使用 C++ 的桌面开发
#      - MSVC v143 生成工具
#      - ARM64/ARM64EC 生成工具（交叉编译 ARM64 必需）
#      - Windows 11 SDK
#      安装地址：https://visualstudio.microsoft.com/visual-cpp-build-tools/
#   2) Rust（MSVC 工具链）：https://rustup.rs 下载 rustup-init.exe 默认安装即可
#
# 用法：
#   powershell -ExecutionPolicy Bypass -File scripts/build-windows.ps1
#   powershell -ExecutionPolicy Bypass -File scripts/build-windows.ps1 -Arch arm64
#   powershell -ExecutionPolicy Bypass -File scripts/build-windows.ps1 -TargetDir D:\rust-target
#
# 注意：项目若在 Z: 这类网络/只读挂载盘上，必须先复制到本地磁盘再构建
#      （构建需要写入 dist/、src-tauri/gen/）

param(
    [ValidateSet("all", "x64", "arm64")]
    [string]$Arch = "all",
    # 编译产物目录默认放本地磁盘：项目在 Z: 网络挂载盘上，放项目内会非常慢
    [string]$TargetDir = "C:\rust-target"
)

$ErrorActionPreference = "Stop"

$project = (Resolve-Path (Join-Path $PSScriptRoot "..\weasel_vision")).Path
$drive = (Get-Item $project).PSDrive
if ($drive.DisplayRoot) {
    Write-Warning "项目位于网络挂载盘 $($drive.Name): ($($drive.DisplayRoot))，构建将无法写入，请先把项目复制到本地磁盘（如 C:\work\Rime_vision）再执行。"
}
Push-Location $project

try {
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        throw "未找到 cargo，请先安装 Rust：https://rustup.rs"
    }

    Write-Host "==> rustc: $(rustc -V)" -ForegroundColor Cyan
    Write-Host "==> 目标目录: $TargetDir" -ForegroundColor Cyan

    Write-Host "==> 安装编译目标" -ForegroundColor Cyan
    rustup target add x86_64-pc-windows-msvc
    rustup target add aarch64-pc-windows-msvc

    Write-Host "==> 安装前端依赖" -ForegroundColor Cyan
    npm install
    if ($LASTEXITCODE -ne 0) { throw "npm install 失败" }

    $env:CARGO_TARGET_DIR = $TargetDir

    if ($Arch -eq "all" -or $Arch -eq "x64") {
        Write-Host "==> 构建 x64 (x86_64-pc-windows-msvc)" -ForegroundColor Green
        npx tauri build --target x86_64-pc-windows-msvc
        if ($LASTEXITCODE -ne 0) { throw "x64 构建失败" }
    }

    if ($Arch -eq "all" -or $Arch -eq "arm64") {
        Write-Host "==> 构建 ARM64 (aarch64-pc-windows-msvc)" -ForegroundColor Green
        npx tauri build --target aarch64-pc-windows-msvc
        if ($LASTEXITCODE -ne 0) { throw "ARM64 构建失败" }
    }

    Write-Host "`n==> 构建完成，安装包位置：" -ForegroundColor Green
    Get-ChildItem -Path $TargetDir -Recurse -Include *.exe, *.msi -ErrorAction SilentlyContinue |
        Where-Object { $_.FullName -match "bundle" } |
        Select-Object FullName, @{n = "MB"; e = { [math]::Round($_.Length / 1MB, 1) } }
}
finally {
    Pop-Location
}
