const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('fs');
const path = require('path');
const { runCommand } = require('./utils');

test('runCommand rejects suspicious arguments', () => {
  const tmpFile = path.join(__dirname, 'hacked.txt');
  fs.rmSync(tmpFile, { force: true });
  assert.throws(
    () => runCommand('echo', ['hello', '&&', 'touch', tmpFile], () => {}),
    TypeError,
  );
  assert.ok(!fs.existsSync(tmpFile));
});
