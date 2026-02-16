#!/usr/bin/env bash

cd "$(dirname "${BASH_SOURCE[0]}")/.."

for file in src/intl/locales/*/extracted.json; do
  locale=$(basename "$(dirname "$file")")
  ../vault-web/node_modules/.bin/formatjs compile "$file" --ast --out-file "src/intl/locales/$locale/compiled.json"
done
