#!/usr/bin/env bash
set -euo pipefail

APP_NAME="Multicode"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="$ROOT_DIR/dist"
APP_BUNDLE="$DIST_DIR/${APP_NAME}.app"
BIN_PATH="$ROOT_DIR/target/release/desktop"

rm -rf "$APP_BUNDLE"
mkdir -p "$APP_BUNDLE/Contents/MacOS"
mkdir -p "$APP_BUNDLE/Contents/Resources"

cp "$BIN_PATH" "$APP_BUNDLE/Contents/MacOS/$APP_NAME"

cat > "$APP_BUNDLE/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleExecutable</key><string>Multicode</string>
  <key>CFBundleIdentifier</key><string>com.example.multicode</string>
  <key>CFBundleName</key><string>Multicode</string>
  <key>CFBundleVersion</key><string>0.1.0</string>
</dict>
</plist>
PLIST

if [[ -n "${CODESIGN_IDENTITY:-}" ]]; then
  codesign --deep --force --sign "$CODESIGN_IDENTITY" "$APP_BUNDLE"
fi

# Placeholder for future Sparkle auto-updater integration
# export SPARKLE_ENABLED=0

echo "Created $APP_BUNDLE"
