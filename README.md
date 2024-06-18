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
docker build --build-arg GIT_REVISION=$(git rev-parse --short HEAD) --build-arg GIT_RELEASE=$(git describe --tags --exact-match 2> /dev/null || echo -n '') --target static-stage -t vault-static .

docker run --rm vault-static cat vault-web.tar.gz > vault-web.tar.gz
```

## Development

For development instructions, see the [`vault-wasm` README](./vault-wasm/README.md) and [`vault-web` README](./vault-web/README.md).

## Releases

### Creating a new release

Push all changes to GitHub and make sure that GitHub Actions pass.

Bump `vault-android` and `vault-ios` versions:

`vault-android/app/build.gradle.kts`:

```
-        versionCode = 112001
-        versionName = "0.1.12"
+        versionCode = 113001
+        versionName = "0.1.13"
```

`vault-ios/Vault.xcodeproj/project.pbxproj`:

```
-				CURRENT_PROJECT_VERSION = 112001;
+				CURRENT_PROJECT_VERSION = 113001;
-				MARKETING_VERSION = 0.1.12;
+				MARKETING_VERSION = 0.1.13;
-				CURRENT_PROJECT_VERSION = 112001;
+				CURRENT_PROJECT_VERSION = 113001;
-				MARKETING_VERSION = 0.1.12;
+				MARKETING_VERSION = 0.1.13;
```

Add changes and commit, but do not push yet:

```sh
git add vault-android/app/build.gradle.kts vault-ios/Vault.xcodeproj/project.pbxproj
git commit -m 'Bump version'
```

Create a new tag:

```sh
git tag v0.1.13
```

Build and upload `vault-android`:

```sh
cd vault-android
source .profile
./gradlew clean
./gradlew generateUniFFIBindings
./gradlew cargoBuild
./gradlew bundleRelease
```

Create a new Internal testing release in Google Play Console and upload the file `vault-android/app/build/outputs/bundle/release/app-release.aab`. Then `Promote release` to `Production`. Send the changes for review. Managed publishing is used so that the reviewed changes will need to be published manually.

Build and upload `vault-ios`:

- open `vault-ios/Vault.xcodeproj` in Xcode
- `Product` > `Archive`
- in `Organizer` > `Archives` select the latest Archive and click `Distribute app`
- select `TestFlight & App Store` and click `Distribute`

In App Store Connect wait for the build to be processed. This can take 15 minutes or more so be patient if the latest version does not yet appear between the builds.

When the build is processed, go to TestFlight and add the build to External testing. If the build is not pushed to TestFlight, App Store reviewers might not use the latest version and the review might get rejected.

Go to Distribution and create a new iOS app version. Select the build and send changes for review. The latest version will be released manually. Wait a few hours for the review to be completed.

If any review gets rejected, remove the Git tag (`git tag --delete v0.1.13`), reset the `Bump version` commit (`git reset --hard HEAD~1`) and start over.

When the Google Play and App Store reviews are successfully completed, push the `main` branch and the Git tag:

```sh
git push origin main v0.1.13
```

Wait for GitHub Actions to succeed.

Go to GitHub Releases, edit the generated release description, and Publish the release.

Go to Google Play Publishing overview and Publish the changes.

Go to App Store Connect and Release the version.

Deploy the latest GitHub Release to https://vault.koofr.net

## Reproducibility

Koofr Vault is built using GitHub Actions and published as a GitHub Release. You can download a release `.tar.gz` file and run it locally, or use the extracted files from the release to verify what https://vault.koofr.net is serving. The current deployed Git revision can be found at https://vault.koofr.net/gitrevision.txt


Example:

```sh
mkdir koofr-vault-check
cd koofr-vault-check

wget https://github.com/koofr/vault/releases/download/v0.1.0/vault-web.tar.gz

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

If you have a bug report, please send the report to [support@koofr.net](mailto:support@koofr.net). If you are considering contributing to the Koofr Vault code, please contact us in order to discuss your intended contribution first, as we can not afford the effort to review arbitrary contributions at the moment.

## Authors and acknowledgement

Koofr Vault was developed and designed by the [Koofr team](https://koofr.eu/team/).

We would like to extend a huge thank you to [rclone](https://rclone.org/) for their encryption implementation.

## License

Koofr Vault is licensed under the terms of the [MIT license](./LICENSE).
