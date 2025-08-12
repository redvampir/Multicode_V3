#!/usr/bin/env node
const fs = require('fs-extra');
const path = require('path');
const { createLogger, createSpinner } = require('./utils');

process.env.NODE_ENV = 'development';

async function main() {
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
      if (await fs.pathExists(full)) {
        await fs.remove(full);
        log(`Removed ${full}`);
      }
    }
    const logsDir = path.join(root, 'logs');
    await fs.emptyDir(logsDir);
    log('Logs cleared');
    spinner.succeed('Clean complete');
  } catch (err) {
    spinner.fail('Clean failed');
    log(err.message);
    process.exit(1);
  }
}

main();
