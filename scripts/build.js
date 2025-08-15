#!/usr/bin/env node
process.env.NODE_ENV = 'production';

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

  const log = createLogger('build');
  log(`NODE_ENV=${process.env.NODE_ENV}`);
  const spinner = createSpinner('Building application');
  spinner.start();
  try {
    await runCommand('npm', ['--prefix', 'frontend', 'run', 'build'], log, { env: process.env });
    spinner.succeed('Build completed');
  } catch (err) {
    spinner.fail('Build failed');
    log(err.message);
    try {
      log('Running setup as fallback');
      await runCommand('node', ['scripts/setup.js'], log, { env: process.env });
      await runCommand('npm', ['--prefix', 'frontend', 'run', 'build'], log, { env: process.env });
      spinner.succeed('Build completed after setup');
    } catch (err2) {
      log('Build failed after setup');
      log(err2.message);
      process.exit(1);
    }
  }
}

main();
