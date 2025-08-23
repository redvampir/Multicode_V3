#!/usr/bin/env node
process.on('unhandledRejection', (err) => {
  console.error(err && err.stack ? err.stack : err);
  process.exit(1);
});
const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const rootDir = path.join(__dirname, '..');

function workspacePackagePaths() {
  const rootPkg = JSON.parse(fs.readFileSync(path.join(rootDir, 'package.json'), 'utf8'));
  const patterns = rootPkg.workspaces || [];
  const paths = ['package.json'];
  for (const pattern of patterns) {
    if (pattern.endsWith('/*')) {
      const base = pattern.slice(0, -2);
      const baseDir = path.join(rootDir, base);
      if (!fs.existsSync(baseDir)) continue;
      for (const entry of fs.readdirSync(baseDir)) {
        const rel = path.join(base, entry, 'package.json');
        if (fs.existsSync(path.join(rootDir, rel))) paths.push(rel);
      }
    } else {
      const rel = path.join(pattern, 'package.json');
      if (fs.existsSync(path.join(rootDir, rel))) paths.push(rel);
    }
  }
  return paths;
}

function gitChanged(files) {
  if (files.length === 0) return false;
  const out = execSync(`git status --porcelain ${files.map(f => `'${f}'`).join(' ')}`, { cwd: rootDir })
    .toString()
    .trim();
  return out.length > 0;
}

const scriptFiles = [
  'scripts/setup.js',
  'scripts/dev.js',
  'scripts/build.js',
  'scripts/test.js',
  'scripts/clean.js',
  'scripts/check-env.js',
];

const pkgFiles = workspacePackagePaths();
const pkgChanged = gitChanged(pkgFiles);
const scriptsChanged = gitChanged(scriptFiles);

if (pkgChanged && !scriptsChanged) {
  console.error('package.json modified without updating scripts. Please review files in scripts/.');
  process.exit(1);
}

console.log('Scripts are up to date.');
