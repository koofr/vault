#!/usr/bin/env bash

cd "$(dirname "${BASH_SOURCE[0]}")/.."

for file in src/features/intl/locales/*/extracted.json; do
  locale=$(basename "$(dirname "$file")")
  npm run formatjs -- compile "$file" --ast --out-file "src/features/intl/locales/$locale/compiled.json"
done
