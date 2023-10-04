# vault-web-tests

Playwright tests for `vault-web`.

## Install dependencies

```sh
npm install
npx playwright install
```

## vault-wasm-nodejs

You need to generate `vault-wasm-nodejs` to run the tests. See section `vault-wasm-nodejs` in [`vault-wasm`
README.md](../vault-wasm/README.md) section.

```sh
cd ../vault-wasm
wasm-pack build --target nodejs --out-dir ../vault-web-tests/vault-wasm-nodejs --out-name vault-wasm
./fix-helpers-nodejs.sh ../vault-web-tests/vault-wasm-nodejs
```

## Update configs

```sh
scripts/use-fake-remote.sh ../vault-web/public/config.json
scripts/use-fake-remote.sh ../vault-web/dist/config.json
```

## Run tests

Before running tests you need to make sure `vite` is running in `vault-web`:

```sh
cd ../vault-web
vite
```

Run tests:

```sh
# Runs the end-to-end tests (headless).
npx playwright test

# Runs the end-to-end tests (non-headless).
npx playwright test --headed

# Starts the interactive UI mode.
npx playwright test --ui

# Runs the tests only on Desktop Chrome.
npx playwright test --project=chromium

# Runs the tests in a specific file.
npx playwright test example

# Runs the tests in debug mode.
npx playwright test --debug

# Auto generate tests with Codegen.
npx playwright codegen
```
