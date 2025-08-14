#!/usr/bin/env node
const { createLogger, runCommand, createSpinner } = require('./utils');

async function main() {
  const log = createLogger('run-tests');
  const cmds = process.argv.slice(2);
  if (cmds.length === 0) {
    log('No test commands provided');
    return;
  }
  for (const cmd of cmds) {
    const spinner = createSpinner(`Running ${cmd}`);
    spinner.start();
    try {
      await runCommand(cmd, [], log, { env: process.env });
      spinner.succeed(`Passed ${cmd}`);
    } catch (err) {
      spinner.fail(`Failed ${cmd}`);
      log(err.message);
      process.exit(1);
    }
  }
}

main();
