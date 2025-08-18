# Сборка Multicode Desktop

Этот документ описывает, как собрать и упаковать приложение `desktop` для основных операционных систем. Все артефакты сборки сохраняются в каталоге `dist/`.

## Предварительные требования

- установленный инструментарий Rust (`cargo`)
- для упаковки Windows: [Inno Setup](https://jrsoftware.org/isinfo.php)
- для сборки пакета macOS: macOS с установленными инструментами командной строки Xcode
- для создания AppImage: `appimagetool` должен быть доступен в `PATH`

## Сборка бинарного файла

```sh
cargo build --release -p desktop
```

Скомпилированный бинарник будет находиться по пути `target/release/desktop` (или `desktop.exe` на Windows).

## Упаковка

### Windows

1. Убедитесь, что установлен Inno Setup.
2. Запустите скрипт установщика:
   ```sh
   iscc scripts/installer.iss
   ```
3. В каталоге `dist/` появится установщик `MulticodeSetup.exe`.

*В установщик включён заглушенный модуль WinSparkle для будущих автообновлений, сейчас он отключён.*

### macOS

1. Запустите скрипт упаковки:
   ```sh
   bash scripts/macos_bundle.sh
   ```
2. `.app`-пакет будет создан в `dist/Multicode.app`.
3. Установите переменную `CODESIGN_IDENTITY`, чтобы при необходимости подписать пакет.

*Заглушки для автообновлений через Sparkle присутствуют, но отключены.*

### Linux (AppImage)

1. Убедитесь, что установлен `appimagetool`.
2. Запустите скрипт создания AppImage:
   ```sh
   bash scripts/build_appimage.sh
   ```
3. Полученный файл AppImage будет сохранён как `dist/Multicode-x86_64.AppImage`.

*В скрипте предусмотрена заглушка проверки версии для будущего автообновления, но сейчас она отключена.*

## GitHub Actions

Рабочий процесс `.github/workflows/desktop.yml` собирает проект `desktop` на платформах `windows-latest`, `macos-latest` и `ubuntu-latest`, упаковывает описанные выше артефакты и загружает их как артефакты в Actions.
