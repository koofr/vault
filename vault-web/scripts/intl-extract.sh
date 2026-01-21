#!/usr/bin/env bash

cd "$(dirname "${BASH_SOURCE[0]}")/.."

npm run formatjs -- extract 'src/**/*.ts*' --ignore='**/*.d.ts' --out-file src/features/intl/locales/en/extracted.json
