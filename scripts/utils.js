let fs;
try {
  fs = require('fs-extra');
} catch {
  fs = require('fs');
  // polyfill ensureDirSync used below
  fs.ensureDirSync = (dir) => fs.mkdirSync(dir, { recursive: true });
}
const path = require('path');
const { spawn } = require('child_process');

let ora;
try {
  ora = require('ora');
} catch {
  ora = (text) => ({
    start: () => console.log(text),
    succeed: (msg) => console.log(msg || text),
    fail: (msg) => console.log(msg || text),
    stop: () => console.log(text),
    isEnabled: false,
  });
}

function createLogger(name) {
  const logsDir = path.join(__dirname, '..', 'logs');
  fs.ensureDirSync(logsDir);
  const logFile = path.join(logsDir, `${name}.log`);
  return (message) => {
    const line = `[${new Date().toISOString()}] ${message}`;
    fs.appendFileSync(logFile, line + '\n');
    console.log(line);
  };
}

function runCommand(cmd, args = [], log = console.log, options = {}) {
  return new Promise((resolve, reject) => {
    const child = spawn(cmd, args, { shell: true, ...options });
    child.stdout.on('data', (data) => log(data.toString().trim()));
    child.stderr.on('data', (data) => log(data.toString().trim()));
    child.on('close', (code) => {
      code === 0 ? resolve() : reject(new Error(`${cmd} ${args.join(' ')} exited with code ${code}`));
    });
  });
}

function createSpinner(text) {
  const spinner = ora(text);
  if (process.env.CI) spinner.isEnabled = false;
  return spinner;
}

module.exports = { createLogger, runCommand, createSpinner };
