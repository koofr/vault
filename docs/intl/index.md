# Internationalization

This page documents how intl is implemented across Vault components and where
translation files flow through the system.

Related docs:

- `docs/intl/setup-weblate.md`
- `docs/intl/add-new-language.md`

## System overview

Vault has two intl stacks:

- Shared Rust/web stack: `vault-intl` + `vault-core/src/intl` +
  `vault-web/src/features/intl`
- Native mobile string stacks: `vault-ios` (`.xcstrings`) and
  `vault-android` (`strings.xml`)

Core and Web use ICU MessageFormat via FormatJS AST catalogs. iOS and Android
render native localized strings in UI, while still using core locale state for
locale selection/subscriptions.

## Key directories

- `vault-intl`
  Rust intl runtime helpers (formatting, plural rules, language negotiation,
  baked ICU data provider)
- `vault-core/src/intl`
  Core intl state/service/selectors and bundled locale catalogs
- `vault-web/src/features/intl`
  React providers and locale/message loading for authenticated and landing page
  flows
- `vault-ios/VaultCommon/Features/Intl`
  iOS language picker UI bound to `MobileVault` intl subscriptions
- `vault-android/app/src/main/java/net/koofr/vault/features/intl`
  Android locale sync helper + language picker UI

## Core and `vault-intl`

Core authoring uses:

```rust
format_message!(formatter, "message.id", "Description", "Default message", args)
```

At runtime:

- `IntlState` holds locales, default locale, current locale, ownership mode
- `IntlService` initializes default/current formatters and persists locale under
  `vaultIntlCurrentLocale` when ownership is core-managed
- Locale matching uses `vault_intl::negotiate_language(...)`
- `FallbackFormatter` always falls back to default locale catalog

Catalog loading:

- `vault-core/build.rs` reads `vault-core/src/intl/locales/locales.json`
- For each locale, it reads `compiled.json`, validates formatter construction,
  minifies JSON, and generates `OUT_DIR/.../locales.min.json`
- `vault-core/src/intl/locales.rs` includes generated locales at compile time

## Web (`vault-web/src/features/intl`)

Two intl modes exist:

- Authenticated app (`WebVaultIntlProvider`)
  locale and locale list come from wasm/core
  (`intlCurrentLocaleSubscribe`, `intlLocalesSubscribe`)
- Unauthenticated landing flow (`LocalStorageIntlProvider`)
  locale is stored in browser `localStorage` (`vaultIntlCurrentLocale`) and
  initialized from `navigator.languages` using
  `@formatjs/intl-localematcher`

Message loading:

- `getMessages.ts` imports all `./locales/*/compiled.json` via Vite glob
- `react-intl` `IntlProvider` renders those messages
- `DateFnsLocaleProvider` maps core locale to date-fns locale

## iOS (`vault-ios`)

iOS UI strings are authored in `vault-ios/VaultCommon/Resources/Localizable.xcstrings`.

Because Weblate does not support `.xcstrings`, iOS uses a conversion bridge:

- export step: `.xcstrings` -> `vault-ios/res/values*/strings.xml`
- import step: `vault-ios/res/values*/strings.xml` -> `.xcstrings`
- tool: `vault-ios/scripts/xcstrings-convert.py`

Runtime locale behavior:

- iOS initializes `MobileVault` with `IntlOwnership.core(preferredLocales: Locale.preferredLanguages)`
- Current core locale is subscribed and injected into SwiftUI via
  `.environment(\.locale, Locale(identifier: currentLocale.locale))`

## Android (`vault-android`)

Android UI strings are native resources:

- base (en): `vault-android/app/src/main/res/values/strings.xml`
- translations: `vault-android/app/src/main/res/values-*/strings.xml`

Runtime locale behavior:

- Android initializes `MobileVault` with `IntlOwnership.External`
- App locale is managed with `AppCompatDelegate.setApplicationLocales(...)`
- `IntlHelper.updateMobileVaultIntlCurrentLocale(...)` pushes current app
  locales to core using lookup strategy (`intlChangeLocale(Lookup(...))`)
- Supported packaged locales are restricted by
  `vault-android/app/build.gradle.kts` `androidResources.localeFilters`

## Translation file flow

Core/web:

1. English source extraction -> `extracted.json`
2. Weblate translates per locale by editing `extracted.json`
3. Compile `extracted.json` -> `compiled.json` (FormatJS AST)
4. Runtime reads `compiled.json`

iOS:

1. Xcode generates `Localizable.xcstrings`
2. Export to `vault-ios/res/values*/strings.xml` for Weblate
3. Import translated XML back into `Localizable.xcstrings`

Android:

1. Author/update `vault-android/.../res/values/strings.xml`
2. Weblate manages `values-*/strings.xml`
3. Android resource system resolves at runtime

## Updating translation sources when adding new strings

This workflow is required whenever new translatable strings are introduced.

Run it before merging any code that adds or changes translation keys. The
`main` branch must always keep extracted/source translation files in sync with
the codebase.

Required updates:

1. iOS:
   build the app so Xcode regenerates
   `vault-ios/VaultCommon/Resources/Localizable.xcstrings`
2. Android:
   manually add new base entries to
   `vault-android/app/src/main/res/values/strings.xml`
3. Run `make intl-extract` from repository root
   (this runs core/web/iOS extraction: `intl-core-extract`,
   `intl-web-extract`, and `intl-ios-extract`)

Commit these translation source updates in the same changeset as the
feature/fix that introduced the strings.

## Updating from Weblate translations

Use this workflow when translators have delivered updates in Weblate.

Branch policy:

- Keep `weblate` clean and Weblate-owned (only commits pushed by Weblate).
- Do not add manual compile commits to `weblate`.
- Compile translations on an integration branch that will be used for PR.

Workflow:

1. Fetch latest remote refs:
   `git fetch origin`
2. Create integration branch from `origin/weblate`:
   `git checkout -b update-translations-YYYYMMDD origin/weblate`
3. Rebase integration branch onto latest `origin/main` (no merge commits):
   `git rebase origin/main`
4. Compile catalogs on the integration branch:
   `make intl-compile`
5. Commit compiled artifacts and push integration branch:
   `git add .`
   `git commit -m "intl: compile catalogs from latest weblate translations"`
   `git push -u origin update-translations-YYYYMMDD`
6. Open a PR from the integration branch to `main`

This keeps the Weblate branch machine-managed while ensuring generated intl
artifacts are reviewed in the PR.

## Commands

Top-level helpers:

```sh
make intl-generate-icu-data
make intl-extract
make intl-compile
```

Per-component commands:

```sh
make intl-core-extract
make intl-core-compile

make intl-web-extract
make intl-web-compile

make intl-ios-extract
make intl-ios-compile

# android does not need extract/compile
```

Script mapping:

- Core extract: `vault-core/scripts/intl-extract.sh` (uses `vault-intl-extract`)
- Core compile: `vault-core/scripts/intl-compile.sh` (FormatJS compile to AST)
- Web extract: `vault-web/scripts/intl-extract.sh` (FormatJS extract)
- Web compile: `vault-web/scripts/intl-compile.sh` (FormatJS compile to AST)
- iOS extract/import: `vault-ios/scripts/xcstrings-convert.py`

## Locale source of truth

- Core locales list: `vault-core/src/intl/locales/locales.json`
- Web locales list (used by landing page):
  `vault-web/src/features/intl/locales/locales.json`
- iOS Xcode project: `vault-ios/Vault.xcodeproj/project.pbxproj`
  (`knownRegions`)
- Android packaged locale filters:
  `vault-android/app/build.gradle.kts` (`androidResources.localeFilters`)

For adding a new locale, follow `docs/intl/add-new-language.md` (Weblate setup,
locale registration, platform locale lists, ICU data regeneration, compilation).
