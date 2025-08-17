#!/usr/bin/env bash

LOG_FILE="backend-ci.log"
: > "$LOG_FILE"

pushd backend >/dev/null

set -o pipefail

run() {
  local cmd="$1"
  echo "+ $cmd" | tee -a "../$LOG_FILE"
  bash -c "$cmd" 2>&1 | tee -a "../$LOG_FILE" || echo "::error file=$(pwd)/../$LOG_FILE,line=1::${cmd} failed" >> "../$LOG_FILE"
}

run "cargo fmt --all -- --check"
run "cargo clippy --all-targets --all-features -- -D warnings"
run "cargo doc --no-deps"
run "cargo audit"
run "cargo build --verbose"
run "cargo build --release"
run "cargo test --all-features -- --nocapture"

popd >/dev/null

tee "$LOG_FILE"
if grep -q "::error" *.log; then
  echo "Some steps failed. See log."
fi
exit 0
