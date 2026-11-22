# WeaselVision GPUI 版 Windows 构建脚本（x64 + ARM64 / NSIS 安装包）
#
# 与 build-windows.ps1（Tauri 版）并存；GPUI 版无需 Node/npm，直接 cargo 构建。
#
# 前置条件（只需装一次）：
#   1) Visual Studio 生成工具 2022，勾选：
#      - 使用 C++ 的桌面开发
#      - MSVC v143 生成工具
#      - ARM64/ARM64EC 生成工具（交叉编译 ARM64 必需）
#      - Windows 11 SDK
#      安装地址：https://visualstudio.microsoft.com/visual-cpp-build-tools/
#   2) Rust（MSVC 工具链）：https://rustup.rs 下载 rustup-init.exe 默认安装即可
#      （项目 rust-toolchain.toml 会自动拉取 1.100）
#
# 用法：
#   powershell -ExecutionPolicy Bypass -File scripts/build-windows-gpui.ps1
#   powershell -ExecutionPolicy Bypass -File scripts/build-windows-gpui.ps1 -Arch arm64
#   powershell -ExecutionPolicy Bypass -File scripts/build-windows-gpui.ps1 -Portable  # 免安装绿色 zip
#
# 注意：项目若在 Z: 这类网络/只读挂载盘上，必须先复制到本地磁盘再构建。

param(
    [ValidateSet("all", "x64", "arm64")]
    [string]$Arch = "all",
    # 出绿色便携 zip（单 exe）而非 NSIS 安装器
    [switch]$Portable,
    [string]$TargetDir = "C:\rust-target"
)

$ErrorActionPreference = "Stop"

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$project = (Join-Path $repoRoot "weasel_vision")
$version = "0.5.0"
$outDir = Join-Path $project "dist"

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
    if ($Arch -eq "all" -or $Arch -eq "x64") { rustup target add x86_64-pc-windows-msvc }
    if ($Arch -eq "all" -or $Arch -eq "arm64") { rustup target add aarch64-pc-windows-msvc }

    $env:CARGO_TARGET_DIR = $TargetDir
    New-Item -ItemType Directory -Force -Path $outDir | Out-Null

    $targets = @()
    if ($Arch -eq "all" -or $Arch -eq "x64") { $targets += @{ Name = "x64"; Triple = "x86_64-pc-windows-msvc" } }
    if ($Arch -eq "all" -or $Arch -eq "arm64") { $targets += @{ Name = "arm64"; Triple = "aarch64-pc-windows-msvc" } }

    foreach ($t in $targets) {
        Write-Host "==> 构建 $($t.Name) ($($t.Triple))" -ForegroundColor Green
        cargo build --release -p weasel-gpui --target $t.Triple
        if ($LASTEXITCODE -ne 0) { throw "$($t.Name) 构建失败" }

        $bin = Join-Path $TargetDir "$($t.Triple)\release\weasel-gpui.exe"
        if (-not (Test-Path $bin)) { throw "未找到产物：$bin" }

        if ($Portable) {
            # 绿色便携版：单 exe 直接可用
            $dst = Join-Path $outDir "WeaselVision_${version}_Windows_$($t.Name).zip"
            Compress-Archive -Path $bin -DestinationPath $dst -Force
            Write-Host "    -> $dst" -ForegroundColor Yellow
        } else {
            # NSIS 安装器：cargo-packager 打包后统一重命名为发布惯例名
            cargo install cargo-packager --locked
            if ($LASTEXITCODE -ne 0) { throw "cargo-packager 安装失败" }
            cargo packager -p weasel-gpui --release --target $t.Triple --bundles nsis
            if ($LASTEXITCODE -ne 0) { throw "$($t.Name) NSIS 打包失败" }
            $nsis = Get-ChildItem -Path (Join-Path $TargetDir "$($t.Triple)\release\bundle") -Recurse -Include *.exe |
                Where-Object { $_.FullName -match "nsis" } |
                Select-Object -First 1
            if (-not $nsis) { throw "未找到 NSIS 安装包输出" }
            $dst = Join-Path $outDir "WeaselVision_${version}_Windows_$($t.Name).exe"
            Copy-Item $nsis.FullName $dst -Force
            Write-Host "    -> $dst" -ForegroundColor Yellow
        }
    }

    Write-Host "`n==> 构建完成，产物位置：" -ForegroundColor Green
    Get-ChildItem -Path $outDir -Include *.exe, *.zip -Recurse -ErrorAction SilentlyContinue |
        Select-Object FullName, @{n = "MB"; e = { [math]::Round($_.Length / 1MB, 1) } }
}
finally {
    Pop-Location
}
