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

See https://v2.tauri.app/start/prerequisites/#windows

### macOS

```sh
xcode-select --install
```

### Linux

See https://v2.tauri.app/start/prerequisites/#linux

```sh
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

## Run

```sh
npm run tauri dev
```

## Build

```sh
npm run tauri build
```
