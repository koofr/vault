# deps

deps: deps-rust deps-web deps-web-tests

deps-rust:
	cargo check

deps-web:
	cd vault-web && npm ci

deps-web-tests:
	cd vault-web-tests && npm ci && npx playwright install

# run

run-web:
	cd vault-web && node_modules/.bin/vite

run-fake-remote:
	cargo run --bin fake_remote

# build

build: build-web build-wasm-web build-wasm-web-tests

build-web: build-wasm-web
	cd vault-web && node_modules/.bin/vite build

build-wasm-web:
	cd vault-wasm && wasm-pack build --target web --out-dir ../vault-web/src/vault-wasm --out-name vault-wasm

build-wasm-web-dev-watch:
	cd vault-wasm && cargo watch -w . -i .gitignore -w ../vault-core -i ../vault-core/.gitignore -s "wasm-pack build --target web --out-dir ../vault-web/src/vault-wasm --out-name vault-wasm --dev"

build-wasm-web-tests:
	cd vault-wasm && wasm-pack build --target nodejs --out-dir ../vault-web-tests/vault-wasm-nodejs --out-name vault-wasm && ./fix-helpers-nodejs.sh ../vault-web-tests/vault-wasm-nodejs

build-ios-simulator:
	cd vault-ios && xcodebuild -scheme Vault -destination "platform=iOS Simulator,name=iPhone 14 Pro" build

build-ios-device:
	cd vault-ios && xcodebuild -scheme Vault -destination "generic/platform=iOS" build

build-ios-archive:
	cd vault-ios && xcodebuild -scheme Vault archive

build-android-bindings: check-android-env
	cd vault-android && ./gradlew generateUniFFIBindings

build-android-library-debug: check-android-env
	cd vault-android && GRADLE_CARGO_PROFILE=debug ./gradlew cargoBuild

build-android-library-release: check-android-env
	cd vault-android && GRADLE_CARGO_PROFILE=release ./gradlew cargoBuild

build-android-assemble-debug: build-android-bindings build-android-library-debug
	cd vault-android && ./gradlew assembleDebug

build-android-assemble-release: build-android-bindings build-android-library-release
	cd vault-android && ./gradlew assembleRelease

build-android-bundle-release: build-android-bindings build-android-library-release
	cd vault-android && ./gradlew bundleRelease

build-fake-remote:
	cargo build --bin fake_remote

# format

format: format-rust format-web format-web-tests format-ios format-android

format-rust:
	cargo +nightly fmt -- --config-path rustfmt-unstable.toml

format-web:
	cd vault-web && npm run prettier

format-web-tests:
	cd vault-web-tests && npm run prettier

format-ios:
	cd vault-ios && swift-format --in-place --recursive .

format-android:
	cd vault-android && ktlint -F app/src

# check

check: check-rust check-web check-web-tests check-android

check-rust:
	cargo check

check-rust-force:
	fd -E target -g '*.rs' -x touch
	make check-rust

check-web: build-wasm-web
	cd vault-web && npm run tsc
	cd vault-web && npm run eslint

check-web-tsc-watch:
	cd vault-web && npm run tsc-watch

check-web-tests: build-wasm-web-tests
	cd vault-web-tests && npm run tsc
	cd vault-web-tests && npm run eslint

check-web-tests-tsc-watch:
	cd vault-web-tests && npm run tsc-watch

check-android-env:
	if [[ -z "$$ANDROID_HOME" ]]; then echo "ANDROID_HOME not set. run source vault-android/.profile"; false; fi

check-android:
	cd vault-android && ktlint app/src

# test

test: test-rust test-web-tests test-ios-unit test-ios-ui test-android-unit test-android-ui

test-rust:
	cargo test

test-rust-force:
	fd -E target -g '*.rs' -x touch
	make test-rust

test-web-tests-prepare: build-wasm-web build-wasm-web-tests
	cd vault-web-tests && scripts/use-fake-remote.sh ../vault-web/public/config.json && scripts/use-fake-remote.sh ../vault-web/dist/config.json

test-web-tests: test-web-tests-prepare
	cd vault-web-tests && npx playwright test --project=chromium

test-web-tests-headed: test-web-tests-prepare
	cd vault-web-tests && npx playwright test --headed --project=chromium

test-ios-unit:
	cd vault-ios && xcodebuild test -scheme Vault -testPlan VaultTests -destination "platform=iOS Simulator,name=iPhone 15 Pro"

test-ios-ui:
	cd vault-ios && xcodebuild test -scheme Vault -testPlan VaultUITests -destination "platform=iOS Simulator,name=iPhone 15 Pro"

test-android-unit: check-android-env build-android-bindings
	cd vault-android && ./gradlew test

test-android-ui: check-android-env build-android-bindings build-android-library-debug
	cd vault-android && ./gradlew connectedAndroidTest
