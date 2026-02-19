#!/usr/bin/env bash

set -euo pipefail

shopt -s nullglob

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
CORE_DIR="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
VAULT_INTL="${CORE_DIR}/../vault-intl"

if [ ! -d "${VAULT_INTL}" ]; then
  echo "vault-intl directory not found"
  exit 1
fi

cd "${CORE_DIR}"

locale_files=(src/intl/locales/*/extracted.json)

if [ "${#locale_files[@]}" -eq 0 ]; then
  echo "No locales found"
  exit 1
fi

locales=()
for file in "${locale_files[@]}"; do
  locales+=("$(basename "$(dirname "$file")")")
done

cd "${VAULT_INTL}"

if ! command -v icu4x-datagen >/dev/null 2>&1; then
  echo "Installing icu4x-datagen..."

  cargo install icu4x-datagen
fi

echo "Generating icu data for locales: ${locales[*]}"

rm -rf src/icu_data

icu4x-datagen \
  --markers PluralsCardinalV1 LocaleLikelySubtagsLanguageV1 LocaleLikelySubtagsScriptRegionV1 LocaleParentsV1 \
  --locales "${locales[@]}" \
  --format baked \
  --pretty \
  --out src/icu_data \
  --use-separate-crates \
  --no-internal-fallback \
  --deduplication maximal
