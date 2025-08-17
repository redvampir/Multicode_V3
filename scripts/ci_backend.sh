#!/usr/bin/env bash

LOG_FILE="$PWD/backend-ci.log"
: > "$LOG_FILE"

pushd backend >/dev/null

FAILED=0

run() {
  local cmd="$1"
  echo "+ $cmd" | tee -a "$LOG_FILE"
  if bash -c "$cmd" 2>&1 | tee -a "$LOG_FILE"; then
    echo "✓ $cmd succeeded" | tee -a "$LOG_FILE"
  else
    echo "✗ $cmd failed" | tee -a "$LOG_FILE"
    FAILED=1
  fi
}

run "cargo fmt --all -- --check"
run "cargo clippy --all-targets --all-features -- -D warnings"
run "cargo doc --no-deps"
run "cargo audit"
run "cargo build --verbose"
run "cargo build --release"
run "cargo test --all-features -- --nocapture"

popd >/dev/null

cat "$LOG_FILE"
if [ "$FAILED" -ne 0 ]; then
  echo "One or more steps failed" | tee -a "$LOG_FILE"
fi
exit "$FAILED"
