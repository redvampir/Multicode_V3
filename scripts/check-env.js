#!/usr/bin/env node
process.on('unhandledRejection', (err) => {
  console.error(err && err.stack ? err.stack : err);
  process.exit(1);
});
const { execSync } = require('child_process');

const isLinux = process.platform === 'linux';

function has(command, label) {
  try {
    execSync(command, { stdio: 'ignore' });
    console.log(`${label}: OK`);
    return true;
  } catch (err) {
    console.error(`${label}: NOT FOUND`);
    return false;
  }
}

function run(command, label) {
  try {
    execSync(command, { stdio: 'inherit' });
    console.log(`${label}: OK`);
    return true;
  } catch (err) {
    console.error(`${label}: FAILED`);
    return false;
  }
}

let ok = true;
ok &= has('pkg-config --version', 'pkg-config');
if (isLinux) {
  ok &= has('pkg-config --exists gdk-3.0', 'gdk-3.0');
  ok &= has('pkg-config --exists gtk+-3.0', 'gtk+-3.0');
} else {
  console.log('Skipped on non-Linux platforms');
}

if (!ok) {
  console.error('\nMissing required system dependencies.');
  process.exit(1);
}

console.log('\nAll required system dependencies found.');

ok &= run('npm audit --audit-level=high', 'npm audit');
ok &= run('cargo audit', 'cargo audit');

if (!ok) {
  console.error('\nDependency audit failed.');
  process.exit(1);
}

console.log('\nDependency audit passed.');

