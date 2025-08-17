#!/usr/bin/env bash

LOG_FILE="backend-ci.log"
: > "$LOG_FILE"

pushd backend >/dev/null

run() {
  local cmd="$1"
  echo "+ $cmd" >> "../$LOG_FILE"
  bash -c "$cmd" >> "../$LOG_FILE" 2>&1 || echo "::error file=backend step=$cmd::failed" >> "../$LOG_FILE"
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
exit 0
