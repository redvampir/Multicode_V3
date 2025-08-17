#!/usr/bin/env bash

LOG_FILE="$PWD/backend-ci.log"
: > "$LOG_FILE"

pushd backend >/dev/null

set -o pipefail
FAILED=0

run() {
  local cmd="$1"
  echo "+ $cmd" | tee -a "$LOG_FILE"
  if ! bash -c "$cmd" 2>&1 | tee -a "$LOG_FILE"; then
    echo "::error file=$LOG_FILE,line=1::${cmd} failed" >> "$LOG_FILE"
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
  echo "::error::One or more steps failed (see log above)"
fi
if grep -q "::error" "$LOG_FILE"; then
  echo "Some steps failed. See log."
fi
exit $FAILED
