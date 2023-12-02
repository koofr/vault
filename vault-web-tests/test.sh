#!/bin/bash

set -e

cd "$(dirname "${BASH_SOURCE[0]}")"/..

echo '##### build web'

cd vault-wasm

wasm-pack build --target web --out-dir ../vault-web/src/vault-wasm --out-name vault-wasm

wasm-pack build --target nodejs --out-dir ../vault-web-tests/vault-wasm-nodejs --out-name vault-wasm

./fix-helpers-nodejs.sh ../vault-web-tests/vault-wasm-nodejs

cd ..

echo '##### build web'

cd vault-web

node_modules/.bin/vite build

cd ..

if ! curl -s http://127.0.0.1:3080/health >/dev/null; then
  echo '##### build and run fake remote'

  cargo build --bin fake_remote

  target/debug/fake_remote &
  fake_remote_pid=$!

  while ! curl -s http://127.0.0.1:3080/health >/dev/null; do
    echo "Waiting for fake remote to start..."
    sleep 1
  done
fi

echo '##### run tests'

cd vault-web-tests

scripts/use-fake-remote.sh ../vault-web/public/config.json
scripts/use-fake-remote.sh ../vault-web/dist/config.json

CI=true npx playwright test --project=chromium
test_exit_code=$?

if [ -n "$fake_remote_pid" ]; then
  echo '##### stop fake remote'

  kill "$fake_remote_pid"
fi

echo '##### exit'

exit $test_exit_code
