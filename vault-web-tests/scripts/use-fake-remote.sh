#!/bin/sh

if [ ! -f "$1" ]; then
  echo "Missing config.json path"
  exit 1
fi

set -e

cp "$1" "$1.bak"

sed 's@"baseUrl":.*@"baseUrl": "https://127.0.0.1:3443",@' "$1.bak" >"$1"

rm "$1.bak"
