# Multicode Desktop Builds

This document describes how to build and package the `desktop` application for the major operating systems. All build artifacts are written to the `dist/` directory.

## Prerequisites

- Rust toolchain (`cargo`) installed
- For Windows packaging: [Inno Setup](https://jrsoftware.org/isinfo.php)
- For macOS bundling: macOS with Xcode command line tools
- For AppImage creation: `appimagetool` available in `PATH`

## Building the binary

```sh
cargo build --release -p desktop
```

The compiled binary will be located at `target/release/desktop` (or `desktop.exe` on Windows).

## Packaging

### Windows

1. Ensure Inno Setup is installed.
2. Run the installer script:
   ```sh
   iscc scripts/installer.iss
   ```
3. The installer `MulticodeSetup.exe` will appear in `dist/`.

_The installer contains a placeholder for future WinSparkle integration. Auto-updates are currently disabled._

### macOS

1. Run the bundling script:
   ```sh
   bash scripts/macos_bundle.sh
   ```
2. The `.app` bundle will be created at `dist/Multicode.app`.
3. Set `CODESIGN_IDENTITY` to sign the bundle if desired.

_Sparkle auto-updater hooks are stubbed out and disabled._

### Linux (AppImage)

1. Ensure `appimagetool` is installed.
2. Run the AppImage script:
   ```sh
   bash scripts/build_appimage.sh
   ```
3. The resulting AppImage will be saved as `dist/Multicode-x86_64.AppImage`.

_A placeholder for a future version check auto-updater is included but disabled._

## GitHub Actions

The workflow defined in `.github/workflows/desktop.yml` builds the `desktop` project on `windows-latest`, `macos-latest` and `ubuntu-latest`, packaging the artifacts described above and uploading them for download.
