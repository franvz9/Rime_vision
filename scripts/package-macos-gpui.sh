#!/bin/bash
# WeaselVision GPUI 版 macOS 打包：.app 组装 + ad-hoc 签名 + hdiutil dmg
# 本机与 CI 共用。用法：package-macos-gpui.sh <版本号> <arch>
#   arch = arm64（原生或交叉，target/aarch64-apple-darwin/release/）
#          x64 （交叉，target/x86_64-apple-darwin/release/）
set -euo pipefail

VER="${1:?usage: package-macos-gpui.sh <version> <arch>}"
ARCH="${2:?usage: package-macos-gpui.sh <version> <arch>}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$SCRIPT_DIR/../weasel_vision"
case "$ARCH" in
  arm64)
    if [ -x "$ROOT/target/aarch64-apple-darwin/release/weasel-gpui" ]; then
      BIN="$ROOT/target/aarch64-apple-darwin/release/weasel-gpui"
    else
      BIN="$ROOT/target/release/weasel-gpui"
    fi ;;
  x64)  BIN="$ROOT/target/x86_64-apple-darwin/release/weasel-gpui" ;;
  *) echo "arch must be arm64 or x64"; exit 1 ;;
esac
DIST="$ROOT/dist"
APP="WeaselVision.app"
DMG="WeaselVision_${VER}_${ARCH}.dmg"
# CI 无 trash 命令；统一挪进一次性临时目录（等价删除，不使用 rm）
TRASH="$(mktemp -d)"

[ -x "$BIN" ] || { echo "binary not found: $BIN"; exit 1; }
mkdir -p "$DIST"

# 旧产物挪进临时目录
for old in "$DIST/$APP" "$DIST/$DMG" "$DIST/dmg-staging"; do
  [ -e "$old" ] && mv "$old" "$TRASH/" 2>/dev/null || true
done

# ── 组装 .app ──
mkdir -p "$DIST/$APP/Contents/MacOS" "$DIST/$APP/Contents/Resources"
cp "$BIN" "$DIST/$APP/Contents/MacOS/weasel-gpui"
cp "$ROOT/assets/icons/icon.icns" "$DIST/$APP/Contents/Resources/icon.icns"

cat > "$DIST/$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key><string>zh_CN</string>
  <key>CFBundleName</key><string>WeaselVision</string>
  <key>CFBundleDisplayName</key><string>WeaselVision</string>
  <key>CFBundleIdentifier</key><string>com.rime.weasel-vision</string>
  <key>CFBundleVersion</key><string>${VER}</string>
  <key>CFBundleShortVersionString</key><string>${VER}</string>
  <key>CFBundleExecutable</key><string>weasel-gpui</string>
  <key>CFBundleIconFile</key><string>icon</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
  <key>LSMinimumSystemVersion</key><string>10.15.0</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>NSSupportsAutomaticGraphicsSwitching</key><true/>
</dict>
</plist>
PLIST

# ad-hoc 签名（未上架分发，本机与 CI 均适用）
codesign --force --sign - "$DIST/$APP" 2>/dev/null || echo "warn: ad-hoc sign skipped"

# ── dmg staging：app + Applications 快捷方式 ──
mkdir -p "$DIST/dmg-staging"
mv "$DIST/$APP" "$DIST/dmg-staging/"
ln -s /Applications "$DIST/dmg-staging/Applications"

hdiutil create -volname "WeaselVision ${VER}" \
  -srcfolder "$DIST/dmg-staging" \
  -format UDZO -ov \
  "$DIST/$DMG" | tail -1

# 还原 .app 到 dist 顶层，staging 挪进临时目录
mv "$DIST/dmg-staging/$APP" "$DIST/"
mv "$DIST/dmg-staging" "$TRASH/" 2>/dev/null || true

echo "── 产物 ──"
ls -lh "$DIST/$DMG" "$DIST/$APP/Contents/MacOS/weasel-gpui"
