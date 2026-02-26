## Create a new release

Make sure `fastlane` is installed (e.g. `brew install fastlane`) for building
and publishing Android and iOS apps.

### 1. Push all changes

Make sure all changes (excluding version bump commit) are pushed to GitHub and
GitHub Actions pass.

### 2. Bump version and create a release tag

Run from repository root:

```sh
scripts/bump-version.py
```

Or with explicit version/build:

```sh
scripts/bump-version.py --version 0.1.23 --build 1
```

### 3. Build `vault-ios`

```sh
fastlane ios build
```

### 4. Build `vault-android`

```sh
source vault-android/.profile
fastlane android build
```

### 5. Upload `vault-ios`

```sh
fastlane ios upload
```

In App Store Connect:

- wait for build processing. This can take 15 minutes or more, so be patient if
  the latest version does not yet appear between the builds
- when the build is processed, go to TestFlight and add the build to External
  testing. If the build is not pushed to TestFlight, App Store reviewers might
  not use the latest version and the review might get rejected
- go to Distribution and create a new iOS app version. Select the build and send
  changes for review. The latest version will be released manually. Wait a few
  hours for the review to be completed

### 6. Upload `vault-android`

```sh
fastlane android upload
```

In Google Play Console:

- promote the new release to Production
- send changes for review

Managed publishing is used, so reviewed changes must be published manually (see
step 8).

### 7. If review is rejected

1. Remove the local unpushed `Bump version` commit (stash is needed because
   reset hard will remove all uncommitted changes):

```sh
git stash
git reset --hard HEAD~1
git stash pop
```

2. Implement required fixes.
3. Re-run bump for the same semantic version with increased build:

```sh
scripts/bump-version.py --build 2 --retag
```

4. Build and submit Android and iOS apps (resubmit both, even if one review
   passed, we need the git revision to be the same and to exist on GitHub) for
   re-review.

### 8. Final publish after both reviews pass

Push `main` branch first:

```sh
git push origin main
```

Wait for GitHub Actions to succeed then push the tag:

```sh
git push origin v0.1.23
```

Wait for GitHub Actions to succeed again (tag-triggered workflows) then:

- in GitHub Releases, edit generated release description and publish release
- in Google Play Publishing overview, publish changes
- in App Store Connect, release the version
- deploy latest GitHub Release to https://vault.koofr.net
