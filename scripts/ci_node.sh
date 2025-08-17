#!/usr/bin/env bash
set -e

WORKDIR="$1"
LOG_FILE="$PWD/${WORKDIR}-ci.log"
: > "$LOG_FILE"

pushd "$WORKDIR" >/dev/null

run() {
  local cmd="$1"
  echo "+ $cmd" >> "$LOG_FILE"
  if ! err=$(bash -c "$cmd" 2>&1); then
    echo "$err" >> "$LOG_FILE"
    local path=$(printf '%s\n' "$err" | grep -oE '[^ :]+\.[a-z]+:[0-9]+' | head -n1)
    echo "::error file=${path:-$WORKDIR} step=$cmd::failed" >> "$LOG_FILE"
    return 1
  else
    echo "$err" >> "$LOG_FILE"
  fi
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

