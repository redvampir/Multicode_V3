const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('fs');
const path = require('path');
const { runCommand } = require('./utils');

test('runCommand does not execute suspicious arguments', async () => {
  const tmpFile = path.join(__dirname, 'hacked.txt');
  fs.rmSync(tmpFile, { force: true });
  await runCommand('echo', ['hello', '&&', 'touch', tmpFile], () => {});
  assert.ok(!fs.existsSync(tmpFile));
});
