const test = require('node:test');
const assert = require('node:assert/strict');
const { spawnSync } = require('child_process');

test('unhandled rejection exits with code 1', () => {
  const code = `
    process.on('unhandledRejection', (err) => {
      console.error(err && err.stack ? err.stack : err);
      process.exit(1);
    });
    Promise.reject(new Error('boom'));
  `;
  const result = spawnSync(process.execPath, ['-e', code], { encoding: 'utf8' });
  assert.equal(result.status, 1);
  const output = result.stderr + result.stdout;
  assert.ok(output.includes('boom'));
});
