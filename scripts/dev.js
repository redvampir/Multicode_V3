#!/usr/bin/env node
process.env.NODE_ENV = 'development';
const fs = require('fs');

async function main() {
  let createLogger, runCommand, createSpinner;
  try {
    ({ createLogger, runCommand, createSpinner } = require('./utils'));
  } catch (err) {
    if (err.code === 'MODULE_NOT_FOUND') {
      console.error(
        'Зависимости не установлены. Запустите `npm install` и затем `npm run setup`.'
      );
      process.exit(1);
    }
    throw err;
  }

  const log = createLogger('dev');
  log(`NODE_ENV=${process.env.NODE_ENV}`);

  const cargoTomlPath = 'frontend/src-tauri/../../legacy-backend/Cargo.toml';
  if (!fs.existsSync(cargoTomlPath)) {
    console.error(
      `Не найден файл '${cargoTomlPath}'. Убедитесь, что бэкенд инициализирован.`
    );
    process.exit(1);
  }

  const spinner = createSpinner('Starting development server');
  spinner.start();
  try {
    await runCommand('npm', ['--prefix', 'frontend', 'run', 'dev'], log, { env: process.env });
    spinner.succeed('Development server exited');
  } catch (err) {
    spinner.fail('Dev failed');
    log(err.message);
    try {
      log('Running setup as fallback');
      await runCommand('node', ['scripts/setup.js'], log, { env: process.env });
      await runCommand('npm', ['--prefix', 'frontend', 'run', 'dev'], log, { env: process.env });
    } catch (err2) {
      log('Dev failed after setup');
      log(err2.message);
      process.exit(1);
    }
  }
}

main();

