# Add a new language

First add the new language in Weblate. This will create empty translation files
for the new language in the `weblate` branch.

The following files will be automatically generated:

- `vault-core/src/intl/locales/LOCALE/extracted.json`
  (`LOCALE` will be in BCP 47 style, e.g. `zh-Hans`)
- `vault-web/src/features/intl/locales/LOCALE/extracted.json`
  (`LOCALE` will be in BCP 47 style, e.g. `zh-Hans`)
- `vault-ios/res/values-LOCALE/strings.xml`
  (`LOCALE` will be in BCP 47 style, e.g. `zh-Hans`)
- `vault-android/app/src/main/res/values-LOCALE/strings.xml`
  (`LOCALE` will be in Android style, e.g. `zh-rCN`)

Fetch latest remote refs and create an integration branch from
`origin/weblate`:

```sh
git fetch origin
git checkout -b add-language-LOCALE origin/weblate
git rebase origin/main
```

1. Add the new locale to `vault-core/src/intl/locales/locales.json`

2. Add the new locale to `vault-web/src/features/intl/locales/locales.json`
   (used for landing page where the WASM bundle is not loaded) and
   `vault-web/src/features/intl/getDateFnsLocale.ts`

3. Add the new locale to iOS Xcode project:
   - in Xcode Project Navigator, click the `Vault` project
   - select the `Vault` project (not `Vault` target), open `Info` tab, add the
     new locale to `Localizations`

4. Add the new locale to Android:
   - in `vault-android/app/build.gradle.kts` add the new locale to
     `androidResources.localeFilters`

5. Regenerate ICU data:

   ```sh
   make intl-generate-icu-data
   ```

6. Compile/import the locales:

   ```sh
   make intl-compile
   ```
