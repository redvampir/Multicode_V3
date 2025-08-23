#!/usr/bin/env node
process.on('unhandledRejection', (err) => {
  console.error(err && err.stack ? err.stack : err);
  process.exit(1);
});
const { execSync } = require('child_process');

const isLinux = process.platform === 'linux';

if (isLinux) {
  try {
    execSync('pkg-config --libs gdk-3.0', { stdio: 'ignore' });
  } catch (err) {
    console.error('gdk-3.0 libraries not found. Install Windows prerequisites.');
    process.exit(1);
  }
}
