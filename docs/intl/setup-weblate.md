# Setup Weblate

## Add translation project

```
Project name: Vault
URL slug: vault
Project website: https://vault.koofr.net
Project description: Koofr Vault is a secure password manager.
```

## Add core translation component

```
Component name: core
URL slug: core
Use as a glossary: no
Project: Vault
Source language: English
Version control system: Git
Source code repository: git@github.com:koofr/vault
Branch: main
```

**Choose translation files to import**

[x] File format Format.JS JSON file, File mask vault-core/src/intl/locales/\*/extracted.json

**Create component**

```
Project: Vault
Category: ------
Component name: core
URL slug: core
Version control system: Git
Source code repository: git@github.com:koofr/vault
Branch: main
Repository push URL: git@github.com:koofr/vault
Push branch: weblate
Repository browser: /
File format: Format.JS JSON
File format parameters:
    Sort JSON keys: yes
    JSON indentation: 2
    JSON indentation style: Spaces
    Avoid spaces after separators: no
File mask: vault-core/src/intl/locales/*/extracted.json
Monolingual base language file: vault-core/src/intl/locales/en/extracted.json
Edit base file: no
Intermediate language file: /
Adding new translation: Create new language file
Template for new translations: /
Translation license: MIT License
Language code style: BCP style using hyphen as a separator
Language filter: `^[^.]+$`
Key filter: /
Source language: English
Use as a glossary: no
```

## Add web translation component

```
Component name: web
URL slug: web
Use as a glossary: no
Project: Vault
Source language: English
Version control system: Git
Source code repository: weblate://vault/core
Branch: /
```

**Choose translation files to import**

[x] File format Format.JS JSON file, File mask vault-web/src/features/intl/locales/\*/extracted.json

**Create component**

```
Project: Vault
Category: ------
Component name: web
URL slug: web
Version control system: Git
Source code repository: weblate://vault/core
Branch: /
Repository push URL: /
Push branch: /
Repository browser: /
File format: Format.JS JSON
File format parameters:
    Sort JSON keys: yes
    JSON indentation: 2
    JSON indentation style: Spaces
    Avoid spaces after separators: no
File mask: vault-web/src/features/intl/locales/*/extracted.json
Monolingual base language file: vault-web/src/features/intl/locales/en/extracted.json
Edit base file: no
Intermediate language file: /
Adding new translation: Create new language file
Template for new translations: /
Translation license: MIT License
Language code style: BCP style using hyphen as a separator
Language filter: `^[^.]+$`
Key filter: /
Source language: English
Use as a glossary: no
```

## Add ios translation component

```
Component name: ios
URL slug: ios
Use as a glossary: no
Project: Vault
Source language: English
Version control system: Git
Source code repository: weblate://vault/core
Branch: /
```

**Choose translation files to import**

[x] File format Android String Resource, File mask vault-ios/res/values-\*/strings.xml

**Create component**

```
Project: Vault
Category: ------
Component name: ios
URL slug: ios
Version control system: Git
Source code repository: weblate://vault/core
Branch: /
Repository push URL: /
Push branch: /
Repository browser: /
File format: Android String Resource
File format parameters:
    Include closing tag for blank XML tags: no
File mask: vault-ios/res/values-*/strings.xml
Monolingual base language file: vault-ios/res/values/strings.xml
Edit base file: no
Intermediate language file: /
Adding new translation: Create new language file
Template for new translations: /
Translation license: MIT License
Language code style: BCP style using hyphen as a separator (for iOS we do not use Android language code style)
Language filter: `^[^.]+$`
Key filter: /
Source language: English
Use as a glossary: no
```

## Add android translation component

```
Component name: android
URL slug: android
Use as a glossary: no
Project: Vault
Source language: English
Version control system: Git
Source code repository: weblate://vault/core
Branch: /
```

**Choose translation files to import**

[x] File format Android String Resource, File mask vault-android/app/src/main/res/values-\*/strings.xml

**Create component**

```
Project: Vault
Category: ------
Component name: android
URL slug: android
Version control system: Git
Source code repository: weblate://vault/core
Branch: /
Repository push URL: /
Push branch: /
Repository browser: /
File format: Android String Resource
File format parameters:
    Include closing tag for blank XML tags: no
File mask: vault-android/app/src/main/res/values-*/strings.xml
Monolingual base language file: vault-android/app/src/main/res/values/strings.xml
Edit base file: no
Intermediate language file: /
Adding new translation: Create new language file
Template for new translations: /
Translation license: MIT License
Language code style: Android style
Language filter: `^[^.]+$`
Key filter: /
Source language: English
Use as a glossary: no
```
