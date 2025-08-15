#!/usr/bin/env node
const fs = require('fs');
const fsp = fs.promises;
const path = require('path');
const readline = require('readline');
const { createLogger, runCommand, createSpinner } = require('./utils');

if (!fsp || !fsp.readFile) {
  console.error('Node.js fs.promises API is required to run setup');
  process.exit(1);
}

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

async function pathExists(p) {
  try {
    await fsp.access(p);
    return true;
  } catch {
    return false;
  }
}

async function getWorkspaceDirs(rootDir, patterns) {
  const dirs = [];
  for (const pattern of patterns) {
    if (pattern.endsWith('/*')) {
      const base = pattern.slice(0, -2);
      const baseDir = path.join(rootDir, base);
      if (!(await pathExists(baseDir))) continue;
      const entries = await fsp.readdir(baseDir);
      for (const entry of entries) {
        const wsDir = path.join(baseDir, entry);
        if (
          (await pathExists(path.join(wsDir, 'package.json')))
          && (await fsp.stat(wsDir)).isDirectory()
        ) {
          dirs.push(wsDir);
        }
      }
    } else {
      const wsDir = path.join(rootDir, pattern);
      if (await pathExists(path.join(wsDir, 'package.json'))) {
        dirs.push(wsDir);
      }
    }
  }
  return dirs;
}

async function main() {
  const log = createLogger('setup');
  log(`NODE_ENV=${process.env.NODE_ENV}`);
  const spinner = createSpinner('Checking dependencies');
  spinner.start();
  try {
    const rootDir = path.join(__dirname, '..');
    const pkgPath = path.join(rootDir, 'package.json');
    const rootPkg = JSON.parse(await fsp.readFile(pkgPath, 'utf8'));
    const workspaces = await getWorkspaceDirs(rootDir, rootPkg.workspaces || []);
    workspaces.unshift(rootDir);

    const missing = {};
    for (const wsDir of workspaces) {
      const pkg = JSON.parse(await fsp.readFile(path.join(wsDir, 'package.json'), 'utf8'));
      const deps = { ...(pkg.dependencies || {}), ...(pkg.devDependencies || {}) };
      const missingDeps = Object.keys(deps).filter((dep) => {
        try {
          require.resolve(dep, { paths: [wsDir] });
          return false;
        } catch {
          return true;
        }
      });
      if (missingDeps.length > 0) {
        const rel = path.relative(rootDir, wsDir) || '.';
        missing[rel] = missingDeps;
      }
    }
    spinner.succeed('Dependency check finished');

    if (Object.keys(missing).length > 0) {
      for (const [ws, deps] of Object.entries(missing)) {
        log(`Missing dependencies in ${ws}: ${deps.join(', ')}`);
      }
      let install = true;
      if (!process.env.CI) {
        const answer = await ask('Install missing dependencies? [Y/n] ');
        install = answer === '' || answer === 'y' || answer === 'yes';
      }
      if (install) {
        for (const [ws, deps] of Object.entries(missing)) {
          const wsDir = path.join(rootDir, ws);
          const installSpinner = createSpinner(`Installing dependencies for ${ws}`);
          installSpinner.start();
          try {
            await runCommand('npm', ['install', ...deps], log, { cwd: wsDir });
            installSpinner.succeed(`Dependencies installed for ${ws}`);
          } catch (err) {
            installSpinner.fail(`npm install failed for ${ws}, trying npm ci`);
            try {
              await runCommand('npm', ['ci'], log, { cwd: wsDir });
              installSpinner.succeed(`npm ci succeeded for ${ws}`);
            } catch (err2) {
              installSpinner.fail(`Installation failed for ${ws}`);
              log(err2.message);
              process.exit(1);
            }
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
