#!/usr/bin/env node
const { createLogger, runCommand, createSpinner } = require('./utils');

process.env.NODE_ENV = 'production';

async function main() {
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
