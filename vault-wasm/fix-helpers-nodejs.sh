#!/bin/sh

if [ ! -d "$1" ]; then
  echo "Missing vault-wasm out path"
  exit 1
fi

helpers_path=$(ls $1/snippets/vault-wasm-*/js/helpers.js)
helpers_path_nodejs="${helpers_path}.nodejs"

echo "Fixing $helpers_path"

exports=$(grep '^export function' "$helpers_path" | sed 's/export function //' | sed 's/(.*//')

sed 's/^export function /function /' "$helpers_path" > "$helpers_path_nodejs"

echo >> "$helpers_path_nodejs"

for export in $exports; do
  echo "module.exports.$export = $export;" >> "$helpers_path_nodejs"
done

mv "$helpers_path_nodejs" "$helpers_path"
