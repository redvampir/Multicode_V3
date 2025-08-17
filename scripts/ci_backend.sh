#!/usr/bin/env bash

set -e

LOG_FILE="backend-ci.log"
: > "$LOG_FILE"

pushd backend >/dev/null

run() {
  local cmd="$1"
  echo "+ $cmd" >> "../$LOG_FILE"
  if ! err=$(bash -c "$cmd" 2>&1); then
    echo "$err" >> "../$LOG_FILE"
    local path=$(printf '%s\n' "$err" | grep -oE '[^ :]+\.[a-z]+:[0-9]+' | head -n1)
    echo "::error file=${path:-backend} step=$cmd::failed" >> "../$LOG_FILE"
    return 1
  else
    echo "$err" >> "../$LOG_FILE"
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

tee "$LOG_FILE"
if grep -q "::error" *.log; then
  echo "Some steps failed. See log."
fi
exit 0
