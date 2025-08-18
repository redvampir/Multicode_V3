# Multicode Desktop

Десктопная версия Multicode работает полностью автономно и не требует запуска web‑сервера на `localhost`.

## Разработка

1. Установите [Rust](https://www.rust-lang.org) и [Node.js](https://nodejs.org).
2. В корне проекта выполните:

   ```sh
   npm install
   npm run setup
   ```

3. Запустите приложение:

   ```sh
   cargo run -p desktop
   ```

   При необходимости дополнительные функции ядра можно подключать через флаги, например `--features "git,watch"`.

## Скрипты упаковки

Все артефакты сохраняются в каталоге `dist/`.

### Windows

```sh
iscc scripts/installer.iss
```

### macOS

```sh
bash scripts/macos_bundle.sh
```

### Linux (AppImage)

```sh
bash scripts/build_appimage.sh
```

## Расположение данных во время работы

- Настройки пользователя: системный каталог конфигурации, например `~/.config/multicode/multicode/settings.json`.
- Журналы: системный каталог данных, `multicode/logs/debug.log` (например `~/.local/share/multicode/logs/debug.log`).

Эти пути определяются автоматически через стандартные механизмы ОС.

