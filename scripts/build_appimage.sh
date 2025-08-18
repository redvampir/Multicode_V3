#!/usr/bin/env bash
set -euo pipefail

APP_NAME="Multicode"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="$ROOT_DIR/dist"
APPDIR="$ROOT_DIR/appimage/${APP_NAME}.AppDir"
BIN_PATH="$ROOT_DIR/target/release/desktop"

rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin"
mkdir -p "$APPDIR/usr/share/applications"
mkdir -p "$APPDIR/usr/share/icons/hicolor/256x256/apps"

cp "$BIN_PATH" "$APPDIR/usr/bin/$APP_NAME"

cat > "$APPDIR/$APP_NAME.desktop" <<EOF2
[Desktop Entry]
Type=Application
Name=$APP_NAME
Exec=$APP_NAME
Icon=$APP_NAME
Categories=Utility;
EOF2

if [ -f "$ROOT_DIR/desktop/assets/icon.png" ]; then
  cp "$ROOT_DIR/desktop/assets/icon.png" "$APPDIR/usr/share/icons/hicolor/256x256/apps/$APP_NAME.png"
fi

mkdir -p "$DIST_DIR"

if command -v appimagetool >/dev/null 2>&1; then
  appimagetool "$APPDIR" "$DIST_DIR/${APP_NAME}-x86_64.AppImage"
else
  echo "appimagetool not found; skipping AppImage generation" >&2
fi

# Placeholder for future version check auto-updater
# export APPIMAGE_AUTOUPDATE=0
