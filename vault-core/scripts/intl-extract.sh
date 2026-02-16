#!/usr/bin/env bash

cd "$(dirname "${BASH_SOURCE[0]}")/../.."

cargo run --package vault-intl-extract -- --include '**/*.rs' --exclude 'vault-intl/**/*.rs' --exclude 'target/**/*' --out-file 'vault-core/src/intl/locales/en/extracted.json'
