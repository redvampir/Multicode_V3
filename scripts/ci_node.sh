#!/usr/bin/env bash

WORKDIR="$1"
if [ -z "$WORKDIR" ]; then
  echo "Usage: $0 WORKDIR"
  exit 1
fi
WORKSPACE_SAFE="${WORKDIR//\//-}"
LOG_FILE="$PWD/${WORKSPACE_SAFE}-ci.log"
: > "$LOG_FILE"

pushd "$WORKDIR" >/dev/null

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

run "npm ci"
run "npx tsc --noEmit"
run "npm run lint -- --max-warnings=0"
run "npm run build --verbose"
run "npm test -- --coverage"
if node -e "process.exit((require('./package.json').scripts||{}).e2e ? 0 : 1)"; then
  run "npm run e2e"
else
  echo "Skipping e2e tests" | tee -a "$LOG_FILE"
fi

popd >/dev/null

cat "$LOG_FILE"
if [ "$FAILED" -ne 0 ]; then
  echo "One or more steps failed" | tee -a "$LOG_FILE"
fi
exit "$FAILED"

