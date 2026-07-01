# macOS Release Signing And Notarization

| Field | Value |
| --- | --- |
| Status | Required before public macOS release |
| Owner | Founder / release engineering |
| Last Updated | 2026-07-01 |
| Decision State | Public macOS downloads must be signed and notarized |

## Plain-English Summary

A customer-ready Mac app must open from Finder with a normal double-click.

The current UAT builds are useful for founder testing, but they are not public release builds. They are ad-hoc signed or unsigned development artifacts, so macOS Gatekeeper may block them after download.

That is why the current workaround removes the quarantine flag in Terminal. That workaround is acceptable only for internal testing.

## Product Requirement

Before System Pulse is linked from the public website as a Mac download, the release artifact must be:

- signed with an Apple Developer ID Application certificate;
- notarized by Apple;
- packaged as a signed DMG;
- verified by GitHub Actions;
- tested from a clean download on macOS without using Terminal.

If any of these checks fail, the build is not customer-ready.

## Required Apple Developer Assets

The signed release workflow expects these GitHub Actions secrets:

| Secret | Purpose |
| --- | --- |
| `APPLE_CERTIFICATE` | Base64-encoded `.p12` export of the Developer ID Application certificate. |
| `APPLE_CERTIFICATE_PASSWORD` | Password used when exporting the `.p12` certificate. |
| `KEYCHAIN_PASSWORD` | Temporary keychain password used only inside the GitHub Actions runner. |
| `APPLE_API_KEY` | App Store Connect API key ID used for notarization. |
| `APPLE_API_ISSUER` | App Store Connect issuer ID. |
| `APPLE_API_PRIVATE_KEY` | Full contents of the `AuthKey_*.p8` private key used for notarization. |

Do not commit certificates, private keys, passwords, or Apple account credentials to the repository.

## Workflow

The manual signed-release workflow lives at:

```text
.github/workflows/macos-signed-release.yml
```

It performs these steps:

1. Checks the required Apple release secrets exist.
2. Imports the Developer ID certificate into a temporary macOS keychain.
3. Builds the Tauri desktop app as a DMG.
4. Lets Tauri sign and notarize the app using the Apple credentials.
5. Runs code-signing and Gatekeeper verification.
6. Uploads `system-pulse-macos-signed-release` as the customer-ready macOS artifact.

## Release Gate

A Mac artifact can be treated as customer-ready only when:

- the `macOS Signed Release` workflow passes;
- the uploaded artifact comes from the intended GitHub commit;
- the downloaded DMG opens normally;
- dragging System Pulse into Applications works normally;
- double-clicking `System Pulse.app` opens without a Terminal command;
- the app still shows real local data and current version text.

## Current Internal UAT Path

The existing `Desktop Build` workflow remains useful for fast internal UAT.

Those artifacts should continue to be labelled as internal builds. They should not be linked from the public website as product downloads.

## References

- Apple Developer documentation: Notarizing macOS software before distribution.
- Tauri documentation: macOS signing and notarization.
