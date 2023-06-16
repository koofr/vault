# vault-android

## Setup Rust toolchains

```sh
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
```

## Local properties

Create `local.properties` file (see `local.properties.example` for examples).

## Profile

Create `.profile` file (see `.profile.example` for examples).

## Build from command line

```sh
# configure env (JAVA_HOME, ANDROID_HOME, PATH)
source .profile

# generate rust bindings
./gradlew generateUniFFIBindings

# build rust library (debug)
./gradlew cargoBuild

# build rust library (release)
GRADLE_CARGO_PROFILE=release ./gradlew cargoBuild

# build apk (debug)
./gradlew assembleDebug

# build apk (release)
./gradlew assembleRelease

# check the app size
ls -lh app/build/outputs/apk/release/app-release.apk

# clean
./gradlew clean
```

## Make a release for Play Store

```sh
# clean
./gradlew clean

# generate rust bindings
./gradlew generateUniFFIBindings

# build rust library (release)
GRADLE_CARGO_PROFILE=release ./gradlew cargoBuild

# build app bundle (release)
./gradlew bundleRelease

# check the app size
ls -lh ./app/build/outputs/bundle/release/app-release.aab
```

## Format code

Install `ktlint`:

```sh
curl -sSLO https://github.com/pinterest/ktlint/releases/download/0.49.1/ktlint && chmod a+x ktlint && sudo mv ktlint /usr/local/bin
```

Format code:

```sh
ktlint -F app/src
```

## Emulator

```sh
# list emulators
emulator -list-avds

# start an emulator
emulator @Pixel_4_API_30
```
