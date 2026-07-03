# System Pulse Apple Certification and Distribution

Date: 3 July 2026

## Decision

System Pulse has two distribution lanes:

1. Private UAT beta: use the existing `Desktop Build` workflow and GitHub artifact download.
2. Public Mac download: use Developer ID signing and Apple notarization before publishing the DMG.

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

Apple's current public distribution path for Mac apps outside the Mac App Store is Developer ID signing plus notarization.

Official references:

- Developer ID: https://developer.apple.com/developer-id/
- Notarizing macOS software before distribution: https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution
- Tauri macOS signing and notarization: https://v2.tauri.app/distribute/sign/macos/

## Required GitHub Secrets

Add these in GitHub repository settings before running `macOS Certified Release`:

Required for signing:

- `APPLE_CERTIFICATE`: base64 encoded Developer ID Application `.p12` certificate.
- `APPLE_CERTIFICATE_PASSWORD`: password for the `.p12` certificate.
- `APPLE_SIGNING_IDENTITY`: Developer ID signing identity, usually like `Developer ID Application: Name (TEAMID)`.

Recommended notarization method, App Store Connect API:

- `APPLE_API_KEY`: App Store Connect API key ID.
- `APPLE_API_ISSUER`: App Store Connect issuer ID.
- `APPLE_API_KEY_P8_BASE64`: base64 encoded `.p8` private key file.

Fallback notarization method, Apple ID:

- `APPLE_ID`: Apple ID email.
- `APPLE_PASSWORD`: app-specific password.
- `APPLE_TEAM_ID`: Apple Developer Team ID.

Use either the App Store Connect API secrets or the Apple ID secrets for notarization. Do not put Apple secrets in the repo.

## Workflow Added

`.github/workflows/macos-certified-release.yml` adds a manual GitHub Actions workflow that:

1. Checks the TypeScript desktop app.
2. Regenerates desktop icons.
3. Validates Apple signing and notarization secrets.
4. Removes the ad-hoc `signingIdentity: "-"` setting for the certified build only.
5. Runs `tauri build --bundles app,dmg` on GitHub's macOS runner.
6. Validates the stapled notarization ticket with `xcrun stapler validate`.
7. Uploads `system-pulse-macos-certified` as the release artifact.

## Website Added

The repository now includes a Replit-ready static promo site under `website/`.

Replit runs it with:

```bash
python3 website/server.py
```

The server sends no-store cache headers so Replit previews stay fresh after each pull.

## Next Human Step

Vanessa needs to add the Apple Developer secrets in GitHub, then run the manual `macOS Certified Release` workflow. Once that succeeds, the certified artifact can become the public website download link.
