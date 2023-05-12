# FROM rust:1.69.0-alpine AS wasm-stage
FROM rust@sha256:3dd0bb6f134635fe40dd9c18bd9603f9d90ce3538ac25ae3e69b9b127137acf2 AS wasm-stage
WORKDIR /app

RUN apk add --no-cache musl-dev zip

# dummy cargo install to update the index to speed up future builds
RUN cargo install empty-library || true

RUN cd /tmp \
  && wget https://github.com/rustwasm/wasm-pack/releases/download/v0.10.3/wasm-pack-v0.10.3-x86_64-unknown-linux-musl.tar.gz \
  && tar xf wasm-pack-v0.10.3-x86_64-unknown-linux-musl.tar.gz \
  && mv wasm-pack-v0.10.3-x86_64-unknown-linux-musl/wasm-pack /usr/local/bin/wasm-pack

COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
COPY vault-core/Cargo.toml vault-core/Cargo.toml
COPY vault-core/user-error-derive/Cargo.toml vault-core/user-error-derive/Cargo.toml
COPY vault-wasm/Cargo.toml vault-wasm/Cargo.toml
RUN mkdir vault-core/src \
  && touch vault-core/src/lib.rs \
  && mkdir vault-core/user-error-derive/src \
  && touch vault-core/user-error-derive/src/lib.rs \
  && mkdir vault-wasm/src \
  && touch vault-wasm/src/lib.rs \
  && sed -i 's/# lto = true/lto = true/' Cargo.toml \
  && cd vault-wasm \
  # this also downloads wasm-opt but the version is pinned so it is reproducible
  && wasm-pack build --target web --out-name vault-wasm \
  && cd .. \
  && rm -Rf vault-core vault-wasm

COPY vault-core vault-core
COPY vault-wasm vault-wasm
# cargo does not build files if mtime is older
RUN find vault-core vault-wasm -type f | xargs touch
RUN cd vault-wasm \
  && wasm-pack build --target web --out-name vault-wasm --out-dir vault-wasm-web \
  && wasm-pack build --target nodejs --out-name vault-wasm --out-dir vault-wasm-nodejs \
  && ./fix-helpers-nodejs.sh vault-wasm-nodejs \
  && cd vault-wasm-nodejs \
  && tar cvzpf ../vault-wasm-nodejs.tar.gz .

# FROM node:17-alpine3.14 AS frontend-stage
FROM node@sha256:0eb54d5716d8cf0dd313a8658dae30bf553edcac2d73f85ceee1a78abf7fdaa5 AS frontend-stage
WORKDIR /app
ARG GIT_REVISION=unknown
COPY vault-web/package.json vault-web/package.json
COPY vault-web/package-lock.json vault-web/package-lock.json
RUN cd vault-web && npm ci
COPY vault-web vault-web
COPY --from=wasm-stage /app/vault-wasm/vault-wasm-web vault-web/src/vault-wasm
RUN cd vault-web && node_modules/.bin/tsc
RUN cd vault-web && node_modules/.bin/eslint src
RUN cd vault-web && VITE_GIT_REVISION=${GIT_REVISION} node_modules/.bin/vite build
RUN echo -n ${GIT_REVISION} > vault-web/dist/gitrevision.txt

# FROM busybox:1.34.1 AS static-stage
FROM busybox@sha256:d345780059f4b200c1ebfbcfb141c67212e1ad4ea7538dcff759895bfcf99e6e AS static-stage
COPY --from=frontend-stage /app/vault-web/dist/ /vault-web
RUN cd vault-web && tar cvzpf ../vault-web.tar.gz .
COPY --from=wasm-stage /app/vault-wasm/vault-wasm-nodejs.tar.gz /vault-wasm-nodejs.tar.gz

# FROM caddy:2.6.2-alpine AS caddy-stage
FROM caddy@sha256:7992b931b7da3cf0840dd69ea74b2c67d423faf03408da8abdc31b7590a239a7 AS caddy-stage
WORKDIR /app
COPY --from=frontend-stage /app/vault-web/dist /app/dist
COPY vault-web/Caddyfile .
EXPOSE 5173
CMD ["caddy", "run"]
