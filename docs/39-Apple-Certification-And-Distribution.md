# System Pulse Apple Certification and Distribution

Date: 3 July 2026

## Decision

System Pulse has two distribution lanes:

1. Private UAT beta: use the existing `Desktop Build` workflow and GitHub artifact download.
2. Public Mac release: use Developer ID signing, Apple notarization, stapling, and a GitHub Release DMG asset.

The private beta can stay ad-hoc signed while the app is still being tested. The public download should not ship until the certified macOS workflow succeeds.

## Current Private Beta

Latest verified private Mac beta:

- Build run: https://github.com/vanessanz78/system-pulse/actions/runs/28593751193
- Mac artifact: https://github.com/vanessanz78/system-pulse/actions/runs/28593751193/artifacts/8040385477
- Artifact name: `system-pulse-macos`
- Source commit: `f6e71d1970112350e964b8b2f9c7b71d13adc943`

Because this build is not notarized yet, macOS may quarantine it after download. For UAT only, copy `System Pulse.app` into Applications and run:

```bash
xattr -dr com.apple.quarantine "/Applications/System Pulse.app" && open "/Applications/System Pulse.app"
```

## Apple Certification Path

Apple's public distribution path for Mac apps outside the Mac App Store is Developer ID signing plus notarization and stapling.

Official references:

- Developer ID: https://developer.apple.com/developer-id/
- Notarizing macOS software before distribution: https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution
- Tauri macOS signing and notarization: https://v2.tauri.app/distribute/sign/macos/

## What Vanessa Already Has

Confirmed current status:

- Apple Developer Program membership.
- Team ID: `7NJLZKW7KA`.
- Developer ID Application certificate created.
- Certificate installed in Keychain with private key attached.
- Local certificate file: `/Users/vanessa/Downloads/developerID_application.cer`.
- Local exported certificate bundle: `/Users/vanessa/Documents/SystemPulseDeveloperID.p12`.
- The `.p12` export password, known only to Vanessa.

Do not commit the `.cer`, `.p12`, `.p8`, certificate passwords, Apple passwords, or base64 secret values to the repository.

## Recommended Notarization Method

Use App Store Connect API keys for CI/CD notarization.

Why:

- It is designed for automation.
- It avoids depending on a personal Apple ID login in CI.
- It is easier to revoke and rotate than a personal app-specific password.
- Tauri supports it with `APPLE_API_KEY`, `APPLE_API_ISSUER`, and `APPLE_API_KEY_PATH`.

The workflow stores the `.p8` private key as a GitHub Secret named `APPLE_API_KEY_P8_BASE64`, writes it to a temporary file during the build, and sets `APPLE_API_KEY_PATH` for Tauri.

Apple ID plus app-specific password remains supported as a fallback only.

## Required GitHub Secrets

Add these in GitHub repository settings before running `macOS Certified Release`:

Required for signing:

- `APPLE_TEAM_ID`: `7NJLZKW7KA`.
- `APPLE_CERTIFICATE_P12`: base64 encoded contents of `/Users/vanessa/Documents/SystemPulseDeveloperID.p12`.
- `APPLE_CERTIFICATE_PASSWORD`: password used when exporting `SystemPulseDeveloperID.p12` from Keychain.

Recommended for notarization, App Store Connect API:

- `APPLE_API_KEY`: App Store Connect API key ID.
- `APPLE_API_ISSUER`: App Store Connect issuer ID.
- `APPLE_API_KEY_P8_BASE64`: base64 encoded contents of the downloaded `AuthKey_XXXXXXXXXX.p8` file.

Fallback notarization method, only if API key setup is blocked:

- `APPLE_ID`: Apple Developer account email.
- `APPLE_APP_SPECIFIC_PASSWORD`: app-specific password generated at appleid.apple.com.

## How To Prepare Secret Values Without Printing Them

Copy the `.p12` secret value to the clipboard without displaying it:

```bash
openssl base64 -A -in "$HOME/Documents/SystemPulseDeveloperID.p12" | pbcopy
```

Then paste into the GitHub Secret named `APPLE_CERTIFICATE_P12`.

When the App Store Connect `.p8` key has been downloaded, copy it to the clipboard without displaying it:

```bash
openssl base64 -A -in "$HOME/Downloads/AuthKey_YOURKEYID.p8" | pbcopy
```

Then paste into the GitHub Secret named `APPLE_API_KEY_P8_BASE64`.

## Workflow

`.github/workflows/macos-certified-release.yml` now supports production release builds.

It runs when:

- a GitHub Release is published, or
- it is manually started with `workflow_dispatch`.

The workflow:

1. Resolves a release tag, defaulting to `v` plus the root `package.json` version.
2. Installs dependencies on a GitHub macOS runner.
3. Checks the TypeScript desktop app.
4. Regenerates desktop icons.
5. Validates Apple signing and notarization secrets.
6. Imports the `.p12` into a temporary CI keychain.
7. Detects the Developer ID Application signing identity automatically.
8. Decodes the App Store Connect API key to a temporary `.p8` file, or maps the Apple ID fallback secret.
9. Removes the ad-hoc `signingIdentity: "-"` setting for the certified build only.
10. Runs `tauri build --bundles app,dmg`.
11. Verifies code signing with `codesign`.
12. Verifies stapling with `xcrun stapler validate`.
13. Verifies Gatekeeper assessment with `spctl`.
14. Uploads the certified bundle as a GitHub Actions artifact.
15. Attaches the DMG to the GitHub Release as `System-Pulse-<tag>.dmg`.

## Release Checklist

Before running the production workflow:

- GitHub Secrets are added.
- A release tag is chosen, for example `v0.1.13`.
- The desktop app version in `package.json` and `tauri.conf.json` matches the release intent.
- The website download link is updated to the release page once the first certified release succeeds.

After the workflow succeeds:

- Download the DMG from the GitHub Release.
- Open the DMG on a clean Mac account if possible.
- Drag `System Pulse.app` into Applications.
- Confirm it opens without the damaged-app quarantine workaround.
- Confirm Gatekeeper shows no scary warning.
- Confirm the menu bar/desktop app still reports realistic live values.

## Replit Website

The repository includes a Replit-ready static promo site under `website/`.

Replit runs it with:

```bash
python3 website/server.py
```

The server sends no-store cache headers so Replit previews stay fresh after each pull.
