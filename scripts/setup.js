#!/usr/bin/env node
const fs = require('fs-extra');
const path = require('path');
const readline = require('readline');
const { createLogger, runCommand, createSpinner } = require('./utils');

process.env.NODE_ENV = process.env.NODE_ENV || 'development';

async function ask(question) {
  const rl = readline.createInterface({ input: process.stdin, output: process.stdout });
  return new Promise((resolve) => {
    rl.question(question, (answer) => {
      rl.close();
      resolve(answer.trim().toLowerCase());
    });
  });
}

async function main() {
  const log = createLogger('setup');
  log(`NODE_ENV=${process.env.NODE_ENV}`);
  const spinner = createSpinner('Checking dependencies');
  spinner.start();
  try {
    const pkgPath = path.join(__dirname, '..', 'package.json');
    const pkg = await fs.readJson(pkgPath);
    const deps = { ...(pkg.dependencies || {}), ...(pkg.devDependencies || {}) };
    const missing = Object.keys(deps).filter((dep) => {
      try {
        require.resolve(dep);
        return false;
      } catch {
        return true;
      }
    });
    spinner.succeed('Dependency check finished');

    if (missing.length > 0) {
      log(`Missing dependencies: ${missing.join(', ')}`);
      let install = true;
      if (!process.env.CI) {
        const answer = await ask(`Install missing dependencies (${missing.join(', ')})? [Y/n] `);
        install = answer === '' || answer === 'y' || answer === 'yes';
      }
      if (install) {
        const installSpinner = createSpinner('Installing dependencies');
        installSpinner.start();
        try {
          await runCommand('npm', ['install', ...missing], log);
          installSpinner.succeed('Dependencies installed');
        } catch (err) {
          installSpinner.fail('npm install failed, trying npm ci');
          try {
            await runCommand('npm', ['ci'], log);
            installSpinner.succeed('npm ci succeeded');
          } catch (err2) {
            installSpinner.fail('Installation failed');
            log(err2.message);
            process.exit(1);
          }
        }
      } else {
        log('Installation skipped by user');
      }
    } else {
      log('All dependencies are installed');
    }
  } catch (err) {
    spinner.fail('Setup failed');
    log(err.stack || err.message);
    process.exit(1);
  }
}

main();
