### wasm

# FROM rust:1.75.0-alpine AS wasm-rust-stage
FROM rust@sha256:7cfbe91942d2673d2b5e3bcfa3b0036b8009d822dc3bfeb1a90b2aaea86eca5d AS wasm-rust-stage
WORKDIR /app

ENV CI=true
ENV CARGO_INCREMENTAL=false
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

RUN apk add --no-cache musl-dev zip

RUN rustup target add wasm32-unknown-unknown

RUN cd /tmp \
  && wget https://github.com/LukeMathWalker/cargo-chef/releases/download/v0.1.62/cargo-chef-x86_64-unknown-linux-musl.tar.gz \
  && tar xf cargo-chef-x86_64-unknown-linux-musl.tar.gz \
  && mv cargo-chef /usr/local/bin/cargo-chef

RUN cd /tmp \
  && wget https://github.com/rustwasm/wasm-pack/releases/download/v0.12.1/wasm-pack-v0.12.1-x86_64-unknown-linux-musl.tar.gz \
  && tar xf wasm-pack-v0.12.1-x86_64-unknown-linux-musl.tar.gz \
  && mv wasm-pack-v0.12.1-x86_64-unknown-linux-musl/wasm-pack /usr/local/bin/wasm-pack

FROM wasm-rust-stage AS wasm-chef-planner-stage
COPY . .
RUN sed -i 's/# lto = true/lto = true/' Cargo.toml
RUN cargo chef prepare --recipe-path recipe.json

FROM wasm-rust-stage AS wasm-stage
COPY --from=wasm-chef-planner-stage /app/recipe.json recipe.json
RUN cargo chef cook --recipe-path recipe.json --target wasm32-unknown-unknown --package vault-wasm --release \
  && cd vault-wasm \
  # this downloads wasm-opt but the version is pinned so it is reproducible
  && wasm-pack build --target web --out-name vault-wasm
ARG GIT_REVISION=unknown
ARG GIT_RELEASE=
ENV GIT_REVISION=${GIT_REVISION}
ENV GIT_RELEASE=${GIT_RELEASE}
COPY . .
RUN sed -i 's/# lto = true/lto = true/' Cargo.toml
RUN cd vault-wasm \
  && wasm-pack build --target web --out-name vault-wasm --out-dir vault-wasm-web \
  && wasm-pack build --target nodejs --out-name vault-wasm --out-dir vault-wasm-nodejs \
  && ./fix-helpers-nodejs.sh vault-wasm-nodejs \
  && cd vault-wasm-nodejs \
  && tar cvzpf ../vault-wasm-nodejs.tar.gz .

### frontend

# FROM node:17-alpine3.14 AS frontend-stage
FROM node@sha256:0eb54d5716d8cf0dd313a8658dae30bf553edcac2d73f85ceee1a78abf7fdaa5 AS frontend-stage
WORKDIR /app
ARG GIT_REVISION=unknown
ARG GIT_RELEASE=
COPY vault-web/package.json vault-web/package.json
COPY vault-web/package-lock.json vault-web/package-lock.json
RUN cd vault-web && npm ci
COPY vault-web vault-web
COPY --from=wasm-stage /app/vault-wasm/vault-wasm-web vault-web/src/vault-wasm
RUN cd vault-web && node_modules/.bin/tsc
RUN cd vault-web && node_modules/.bin/eslint src
RUN cd vault-web && VITE_GIT_REVISION=${GIT_REVISION} VITE_GIT_RELEASE=${GIT_RELEASE} node_modules/.bin/vite build
RUN echo -n ${GIT_REVISION} > vault-web/dist/gitrevision.txt
RUN echo -n ${GIT_RELEASE} > vault-web/dist/gitrelease.txt

### static

# FROM busybox:1.34.1 AS static-stage
FROM busybox@sha256:d345780059f4b200c1ebfbcfb141c67212e1ad4ea7538dcff759895bfcf99e6e AS static-stage
COPY --from=frontend-stage /app/vault-web/dist/ /vault-web
RUN cd vault-web && tar cvzpf ../vault-web.tar.gz .
COPY --from=wasm-stage /app/vault-wasm/vault-wasm-nodejs.tar.gz /vault-wasm-nodejs.tar.gz

### caddy

# FROM caddy:2.6.2-alpine AS caddy-stage
FROM caddy@sha256:7992b931b7da3cf0840dd69ea74b2c67d423faf03408da8abdc31b7590a239a7 AS caddy-stage
WORKDIR /app
COPY --from=frontend-stage /app/vault-web/dist /app/dist
COPY vault-web/Caddyfile .
EXPOSE 5173
CMD ["caddy", "run"]
