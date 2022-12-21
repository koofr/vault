![Koofr Vault](./vault-web/src/assets/images/vault-logo.svg)

# Koofr Vault

![build status](https://github.com/koofr/vault/actions/workflows/build.yml/badge.svg)

https://vault.koofr.net

Koofr Vault is an open-source, client-side encrypted folder for your Koofr cloud storage offering an extra layer of security for your most sensitive files. The encryption is compatible with [rclone](https://rclone.org/).

## Tech stack

Koofr Vault is divided into two parts: the engine and the UI. The engine, made up of `vault-core` and `vault-wasm`, is written in Rust and compiled to WebAssembly. The UI, `vault-web`, is written in React and uses Vite for frontend tooling. There is no server component; Koofr Vault only uses the public Koofr REST API.

## Build and run locally

The easiest way to run Koofr Vault locally is to build and run it with Docker. This will compile the app and build a Docker image using [Caddy web server](https://caddyserver.com/).

```sh
docker build --build-arg GIT_REVISION=$(git rev-parse --short HEAD) -t vault .

docker run --rm -p 5173:5173 vault
```

You can now open http://localhost:5173 in your browser.

### Build static files

Alternatively, you can build static files, which will produce `vault-web.tar.gz` file with the static files:

```sh
docker build --build-arg GIT_REVISION=$(git rev-parse --short HEAD) --target static-stage -t vault-static .

docker run --rm vault-static cat vault-web.tar.gz > vault-web.tar.gz
```

## Development

For development instructions, see the [`vault-wasm` README](./vault-wasm/README.md) and [`vault-web` README](./vault-web/README.md).

## Reproducibility

Koofr Vault is built using GitHub Actions and published as a GitHub Release. You can download a release `.tar.gz` file and run it locally, or use the extracted files from the release to verify what https://vault.koofr.net is serving. The current deployed Git revision can be found at https://vault.koofr.net/gitrevision.txt


Example:

```sh
mkdir koofr-vault-check
cd koofr-vault-check

wget https://repo.koofr.eu/vault/vault-web-052a457.tar.gz

tar xf vault-web.tar.gz

allok=true

for path in $(tar tf vault-web.tar.gz | grep -v './$' | sort); do
  local=$(sha256sum "$path" | awk '{print $1}')
  remote=$(curl -s "https://vault.koofr.net/${path:2}" | sha256sum | awk '{print $1}')
  if [ "$local" = "$remote" ]; then
    echo "$path OK"
  else
    echo "$path MISMATCH (local $local remote $remote)"
    allok=false
  fi
done

if [ "$allok" = "true" ]; then
  echo "OK"
else
  echo "MISMATCH"
fi
```

## Contributing

If you have a bug report, please send the report to [support@koofr.net](mailto:support@koofr.net). If you are considering contributing to the Koofr Vault code, please contact as to discuss your intended contribution first, as we can not afford the effort to review arbitrary contributions at the moment.

## Authors and acknowledgement

Koofr Vault was developed and designed by the [Koofr team](https://koofr.eu/team/).

We would like to extend a huge thank you to [rclone](https://rclone.org/) for their encryption implementation.

## License

Koofr Vault is licensed under the terms of the [MIT license](./LIENSE).
