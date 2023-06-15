#!/usr/bin/env bash

cd ../vault-mobile/uniffi-bindgen

CARGO=$HOME/.cargo/bin/cargo

if [[ -n "${DEVELOPER_SDK_DIR:-}" ]]; then
  # Assume we're in Xcode, which means we're probably cross-compiling. In this
  # case, we need to add an extra library search path for build scripts and
  # proc-macros, which run on the host instead of the target. (macOS Big Sur
  # does not have linkable libraries in /usr/lib/.)
  export LIBRARY_PATH="${DEVELOPER_SDK_DIR}/MacOSX.sdk/usr/lib:${LIBRARY_PATH:-}"
fi

$CARGO run generate ../src/vault-mobile.udl --language swift --no-format --out-dir "${PROJECT_DIR}/VaultMobile"
