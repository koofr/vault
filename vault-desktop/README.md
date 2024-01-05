# vault-desktop

Vault desktop app

## Dependencies

Install dependencies:

```sh
npm install
```

Install vault-web dependencies:

```sh
cd ../vault-web
npm install
```

### Windows

See https://tauri.app/v1/guides/getting-started/prerequisites/#setting-up-windows

### macOS

```sh
xcode-select --install
```

### Linux

```sh
sudo apt install libwebkit2gtk-4.0-dev build-essential curl wget file libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
```

## Run

```sh
npm run tauri dev
```

## Build

```sh
npm run tauri build
```
