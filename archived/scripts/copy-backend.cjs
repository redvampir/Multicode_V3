const fs = require('fs');
const path = require('path');
const triple = process.env.TAURI_ENV_TARGET_TRIPLE;
if (!triple) {
  console.error('TAURI_ENV_TARGET_TRIPLE is not set');
  process.exit(1);
}
const ext = process.platform === 'win32' ? '.exe' : '';
const src = path.resolve(__dirname, '../legacy-backend/target/release/backend' + ext);
const dest = path.resolve(__dirname, '../legacy-backend/target/release/backend-' + triple + ext);
fs.copyFileSync(src, dest);
console.log('Copied backend to', dest);
