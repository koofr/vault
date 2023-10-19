# deps

deps: deps-rust deps-web deps-web-tests

deps-rust:
	cargo check

deps-web:
	cd vault-web && npm ci

deps-web-tests:
	cd vault-web-tests && npm ci && npx playwright install

# build

build: build-web build-wasm-web build-wasm-web-tests

build-web: build-wasm-web
	cd vault-web && vite

build-wasm-web:
	cd vault-wasm && wasm-pack build --target web --out-dir ../vault-web/src/vault-wasm --out-name vault-wasm

build-wasm-web-tests:
	cd vault-wasm && wasm-pack build --target nodejs --out-dir ../vault-web-tests/vault-wasm-nodejs --out-name vault-wasm && ./fix-helpers-nodejs.sh ../vault-web-tests/vault-wasm-nodejs

# format

format: format-rust format-web format-web-tests

format-rust:
	cargo +nightly fmt -- --config-path rustfmt-unstable.toml

format-web:
	cd vault-web && npm run prettier

format-web-tests:
	cd vault-web-tests && npm run prettier

# check

check: check-rust check-web check-web-tests

check-rust:
	cargo check

check-rust-force:
	fd -E target -g '*.rs' -x touch
	make check-rust

check-web: build-wasm-web
	cd vault-web && npm run tsc
	cd vault-web && npm run eslint

check-web-tests: build-wasm-web-tests
	cd vault-web-tests && npm run tsc
	cd vault-web-tests && npm run eslint

# test

test: test-rust test-web-tests

test-rust:
	cargo test

test-rust-force:
	fd -E target -g '*.rs' -x touch
	make test-rust

test-web-tests: build-wasm-web build-wasm-web-tests
	cd vault-web-tests && scripts/use-fake-remote.sh ../vault-web/public/config.json && scripts/use-fake-remote.sh ../vault-web/dist/config.json
	cd vault-web-tests && npx playwright test --headed --project=chromium
