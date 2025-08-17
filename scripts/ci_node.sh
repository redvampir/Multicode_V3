#!/usr/bin/env bash

WORKDIR="$1"
WORKSPACE_SAFE="${WORKDIR//\//-}"
LOG_FILE="$PWD/${WORKSPACE_SAFE}-ci.log"
: > "$LOG_FILE"

pushd "$WORKDIR" >/dev/null

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

run "npm ci"
run "npx tsc --noEmit"
run "npm run lint -- --max-warnings=0"
run "npm run build --verbose"
run "npm test -- --coverage"
if node -e "process.exit((require('./package.json').scripts||{}).e2e ? 0 : 1)"; then
  run "npm run e2e"
fi

popd >/dev/null

cat "$LOG_FILE"
if [ "$FAILED" -ne 0 ]; then
  echo "::error::One or more steps failed (see log above)"
fi
if grep -q "::error" "$LOG_FILE"; then
  echo "Some steps failed. See log."
fi
exit $FAILED

