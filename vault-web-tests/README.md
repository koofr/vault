# vault-web-tests

Playwright tests for `vault-web`.

## Auth

To run tests you'll need an OAuth 2 token. Open http://localhost:5173/ in your
browser, sign in with Koofr, open dev console and run the following:

```js
console.log(
  JSON.stringify(
    {
      cookies: [],
      origins: [
        {
          origin: "http://localhost:5173",
          localStorage: [
            {
              name: "vaultOAuth2Token",
              value: localStorage.vaultOAuth2Token,
            },
          ],
        },
      ],
    },
    null,
    2
  )
);
```

Copy and paste the printed JSON into `playwright/.auth/user.json`.

## vault-wasm-nodejs

You need to generate `vault-wasm-nodejs` to run the tests. See section `vault-wasm-nodejs` in [`vault-wasm`
README.md](../vault-wasm/README.md) section.

```sh
cd ../vault-wasm
wasm-pack build --target nodejs --out-dir ../vault-web-tests/vault-wasm-nodejs --out-name vault-wasm
./fix-helpers-nodejs.sh ../vault-web-tests/vault-wasm-nodejs
```

## Run tests

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
