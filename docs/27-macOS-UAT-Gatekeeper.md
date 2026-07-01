# macOS UAT Gatekeeper Notes

System Pulse GitHub Actions builds are private UAT builds until the project has Apple Developer ID signing and notarization configured.

## What Vanessa May See

macOS may show one of these messages after the app is downloaded from GitHub:

- `System Pulse is damaged and can't be opened.`
- `System Pulse cannot be opened because Apple cannot check it for malicious software.`
- `System Pulse Not Opened. Apple could not verify System Pulse is free of malware.`

This does not mean PulseCore or the app data collectors are broken. It means macOS Gatekeeper does not yet trust the downloaded build as a public release.

## Temporary UAT Unblock

For private testing only, after copying the app into Applications:

```bash
xattr -dr com.apple.quarantine "/Applications/System Pulse.app" && open "/Applications/System Pulse.app"
```

This only removes the download quarantine flag from the UAT app. It does not run cleanup, optimise the Mac, restart apps, or change user data.

## Product Requirement Before Public Release

A public System Pulse Mac release must open by double-clicking the app icon. If the user has to run a Terminal command, the build is still internal UAT only.

Before System Pulse is offered from the future public website, macOS distribution must add:

- Apple Developer ID certificate storage in GitHub Actions secrets;
- signed macOS app bundles;
- signed DMG output;
- Apple notarization;
- Gatekeeper verification inside GitHub Actions;
- a release checklist that verifies first-open behavior on a clean macOS user account.

The signed-release workflow is defined in `.github/workflows/macos-signed-release.yml` and documented in [macOS Release Signing And Notarization](34-macOS-Release-Signing-And-Notarization.md).

Unsigned or ad-hoc signed artifacts are acceptable only for internal UAT.