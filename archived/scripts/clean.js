#!/usr/bin/env node
const fs = require('fs');
const fsp = fs.promises;
const path = require('path');
if (!fsp || !fsp.rm) {
  console.error('Node.js fs.promises.rm is required to run clean');
  process.exit(1);
}

process.env.NODE_ENV = 'development';

async function pathExists(p) {
  try {
    await fsp.access(p);
    return true;
  } catch {
    return false;
  }
}

async function emptyDir(dir) {
  await fsp.mkdir(dir, { recursive: true });
  const entries = await fsp.readdir(dir);
  await Promise.all(
    entries.map((entry) => fsp.rm(path.join(dir, entry), { recursive: true, force: true }))
  );
}

async function main() {
  let createLogger, createSpinner;
  try {
    ({ createLogger, createSpinner } = require('./utils'));
  } catch (err) {
    if (err.code === 'MODULE_NOT_FOUND') {
      console.error(
        'Зависимости не установлены. Запустите `npm install` и затем `npm run setup`.'
      );
      process.exit(1);
    }
    throw err;
  }

  const log = createLogger('clean');
  log(`NODE_ENV=${process.env.NODE_ENV}`);
  const spinner = createSpinner('Cleaning artifacts');
  spinner.start();
  const root = path.join(__dirname, '..');
  const targets = [
    'dist',
    'build',
    'node_modules',
    path.join('frontend', 'dist'),
    path.join('frontend', 'build'),
    path.join('frontend', 'node_modules'),
  ];
  try {
    for (const target of targets) {
      const full = path.join(root, target);
      if (await pathExists(full)) {
        await fsp.rm(full, { recursive: true, force: true });
        log(`Removed ${full}`);
      }
    }
    const logsDir = path.join(root, 'logs');
    await emptyDir(logsDir);
    log('Logs cleared');
    spinner.succeed('Clean complete');
  } catch (err) {
    spinner.fail('Clean failed');
    log(err.message);
    process.exit(1);
  }
}

main();
