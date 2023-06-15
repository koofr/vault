#!/usr/bin/env bash

# based on https://fnordig.de/2022/01/31/rust-libraries-on-ios/

if [ "$#" -ne 1 ]
then
    echo "Usage (note: only call inside xcode!):"
    echo "compile-library.sh <buildvariant>"
    exit 1
fi

# buildvariant from our xcconfigs
BUILDVARIANT=$1

RELFLAG=
if [[ "$BUILDVARIANT" != "debug" ]]; then
  RELFLAG=--release
fi

if [[ "$BUILDVARIANT" == "debug" ]]; then
  true

  # uncomment to speed up builds (when rust does not need to recompile)
  # exit
fi

set -euvx

IS_SIMULATOR=0
if [ "${LLVM_TARGET_TRIPLE_SUFFIX-}" = "-simulator" ]; then
  IS_SIMULATOR=1
fi

CARGO=$HOME/.cargo/bin/cargo

export GIT_REVISION=$(git rev-parse --short HEAD)
export GIT_RELEASE=$(git describe --tags --exact-match 2> /dev/null)

cd ../vault-mobile

# We need to set LIBRARY_PATH for build scripts but we cannot use the same
# LIBRARY_PATH for build scripts and for cross-compiling. First we run cargo
# check with LIBRARY_PATH=$HOST_LIBRARY_PATH to run the build scripts and then
# cargo build (which will not rerun build scripts).

# Assume we're in Xcode, which means we're probably cross-compiling. In this
# case, we need to add an extra library search path for build scripts and
# proc-macros, which run on the host instead of the target. (macOS Big Sur does
# not have linkable libraries in /usr/lib/.)
HOST_LIBRARY_PATH="${DEVELOPER_SDK_DIR}/MacOSX.sdk/usr/lib:${LIBRARY_PATH:-}"

rm -f "${PROJECT_DIR}/VaultMobile/libvault_mobile.a"

for arch in $ARCHS; do
  case "$arch" in
    x86_64)
      if [ $IS_SIMULATOR -eq 0 ]; then
        echo "Building for x86_64, but not a simulator build. What's going on?" >&2
        exit 2
      fi

      # Intel iOS simulator
      export CFLAGS_x86_64_apple_ios="-target x86_64-apple-ios"
      LIBRARY_PATH=$HOST_LIBRARY_PATH $CARGO check --lib $RELFLAG --target x86_64-apple-ios
      $CARGO build --lib $RELFLAG --target x86_64-apple-ios
      cp "../target/x86_64-apple-ios/$BUILDVARIANT/libvault_mobile.a" "${PROJECT_DIR}/VaultMobile"
      ;;

    arm64)
      if [ $IS_SIMULATOR -eq 0 ]; then
        # Hardware iOS targets
        LIBRARY_PATH=$HOST_LIBRARY_PATH $CARGO check --lib $RELFLAG --target aarch64-apple-ios
        $CARGO build --lib $RELFLAG --target aarch64-apple-ios
        cp "../target/aarch64-apple-ios/$BUILDVARIANT/libvault_mobile.a" "${PROJECT_DIR}/VaultMobile"
      else
        LIBRARY_PATH=$HOST_LIBRARY_PATH $CARGO check --lib $RELFLAG --target aarch64-apple-ios-sim
        $CARGO build --lib $RELFLAG --target aarch64-apple-ios-sim
        cp "../target/aarch64-apple-ios-sim/$BUILDVARIANT/libvault_mobile.a" "${PROJECT_DIR}/VaultMobile"
      fi
  esac
done
