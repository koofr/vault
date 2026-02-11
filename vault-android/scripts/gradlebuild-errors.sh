#!/usr/bin/env bash

set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

source .profile

./gradlew assembleDebug 2>&1 | grep -E "(FAILED|^e: )" || true

exit ${PIPESTATUS[0]}
