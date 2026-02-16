# vault-core

## Intl

### Regenerate ICU data

```sh
cd ..
make intl-generate-icu-data
```

### Extract translations

```sh
cd ..
make intl-core-extract
```

### Compile translations

```sh
cd ..
make intl-core-compile
```

## Testing

### Code coverage

Install tools:

```sh
cargo install cargo-tarpaulin
```

Run tests:

```sh
cargo tarpaulin -o html
```

Open `../tarpaulin-report.html`
