#!/usr/bin/env bash

set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

xcodebuild -scheme Vault -destination "platform=iOS Simulator,name=iPhone 17 Pro" build 2>&1 \
  | grep -E "(\*\* BUILD FAILED \*\*|fatal error:|\\serror:|Undefined symbols|ld: )" || true

exit ${PIPESTATUS[0]}
