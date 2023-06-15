# vault-ios

## Setup Rust toolchains

```sh
rustup target add aarch64-apple-ios x86_64-apple-ios aarch64-apple-ios-sim
```

## Build from command line

```sh
# list targets, build configurations and schemes
xcodebuild -list -project Vault.xcodeproj

# show destinations
xcodebuild -scheme Vault -showdestinations

# build
xcodebuild -scheme Vault -destination "platform=iOS Simulator,name=iPhone 14 Pro" build

# clean
xcodebuild -scheme Vault clean
```

## Generate bindings manually

```sh
PROJECT_DIR=$(pwd) bin/generate-bindings.sh
```

## Generate release archive

Generate an app archive (e.g. to analyze its size):

```sh
# build
xcodebuild -project Vault.xcodeproj -scheme Vault -destination "generic/platform=iOS" -archivePath Release/Vault.xcarchive archive

# app size
du -hs Release/Vault.xcarchive/Products/Applications/Vault.app

# export
xcodebuild -exportArchive -archivePath Release/Vault.xcarchive -exportPath Release/Vault -exportOptionsPlist ExportOptionsAppStore.plist
```

## Format code

Install `swift-format`:

```sh
brew install mint
mint install apple/swift-format@main
ln -s $HOME/.mint/bin/swift-format /usr/local/bin/swift-format
```

Format code:

```sh
swift-format --in-place --recursive .
```
