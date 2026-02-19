# vault-core

## Intl

### Regenerate ICU data

```sh
cd ..
make intl-generate-icu-data
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
