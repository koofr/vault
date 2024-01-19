# vault-web

`vault-web` contains the Web UI for Vault.

## Develop

### Install dependencies

```sh
npm install
```

### Build the WebAssembly package

Look at the `README.md` in `../vault-wasm`.

```sh
cd ../vault-wasm
wasm-pack build --target web --out-dir ../vault-web/src/vault-wasm --out-name vault-wasm
```

### Run Vite CLI

```sh
vite
```

Open http://localhost:5173 in your browser.

### Check for TypeScript errors

```sh
tsc --watch
# or tsc --watch
```

## Build and run

```sh
vite build
```

Run caddy:

```sh
caddy run
```

## Optimize SVG icons

```sh
find src/assets -name '*.svg' | xargs -n1 scripts/optimize-svg.js
```

## Maintenance

### Upgrade dependencies

```sh
npm update --save
```
