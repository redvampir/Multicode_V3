#!/usr/bin/env node
const { execSync } = require('child_process');

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

let ok = true;
ok &= has('pkg-config --version', 'pkg-config');
ok &= has('pkg-config --exists gdk-3.0', 'gdk-3.0');
ok &= has('pkg-config --exists gtk+-3.0', 'gtk+-3.0');

if (!ok) {
  console.error('\nMissing required system dependencies.');
  process.exit(1);
} else {
  console.log('\nAll required system dependencies found.');
}
