#!/usr/bin/env node
const { createLogger, runCommand, createSpinner } = require('./utils');

process.env.NODE_ENV = 'development';

async function main() {
  const log = createLogger('dev');
  log(`NODE_ENV=${process.env.NODE_ENV}`);
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
