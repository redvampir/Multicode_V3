#!/usr/bin/env bash

WORKDIR="$1"
LOG_FILE="$PWD/${WORKDIR}-ci.log"
: > "$LOG_FILE"

pushd "$WORKDIR" >/dev/null

set -o pipefail

run() {
  local cmd="$1"
  echo "+ $cmd" | tee -a "$LOG_FILE"
  bash -c "$cmd" 2>&1 | tee -a "$LOG_FILE" || echo "::error file=$(pwd)/../${WORKDIR}-ci.log,line=1::${cmd} failed" >> "$LOG_FILE"
}

run "npm ci"
run "npx tsc --noEmit"
run "npm run lint -- --max-warnings=0"
run "npm run build --verbose"
run "npm test -- --coverage"
if node -e "process.exit((require('./package.json').scripts||{}).e2e ? 0 : 1)"; then
  run "npm run e2e"
fi

popd >/dev/null

tee "$LOG_FILE"
if grep -q "::error" *.log; then
  echo "Some steps failed. See log."
fi
exit 0

