# vault-wasm

`vault-wasm` contains the WebAssembly bindings for the Vault core engine.

## Build

Build the WebAssembly package using `wasm-pack`:

```sh
wasm-pack build --target web --out-dir ../vault-web/src/vault-wasm --out-name vault-wasm
```

Watch for changes and build (you need to have [cargo watch](https://crates.io/crates/cargo-watch) installed):

```sh
cargo watch -w . -i .gitignore -w ../vault-core -i ../vault-core/.gitignore -s "wasm-pack build --target web --out-dir ../vault-web/src/vault-wasm --out-name vault-wasm"
```

Watch for changes and run `cargo check`:

```sh
cargo watch -w . -i .gitignore -w ../vault-core -i ../vault-core/.gitignore
```
