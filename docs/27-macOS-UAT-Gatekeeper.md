# macOS UAT Gatekeeper Notes

System Pulse GitHub Actions builds are private UAT builds until the project has Apple Developer ID signing and notarization configured.

## What Vanessa May See

macOS may show one of these messages after the app is downloaded from GitHub:

- `System Pulse is damaged and can't be opened.`
- `System Pulse cannot be opened because Apple cannot check it for malicious software.`

This does not mean PulseCore or the app data collectors are broken. It means macOS Gatekeeper does not yet trust the downloaded build as a public release.

## Temporary UAT Unblock

For private testing only, after copying the app into Applications:

```bash
xattr -dr com.apple.quarantine "/Applications/System Pulse.app" && open "/Applications/System Pulse.app"
```

This only removes the download quarantine flag from the UAT app. It does not run cleanup, optimise the Mac, restart apps, or change user data.

## Product Requirement Before Public Release

Before System Pulse is offered from the future public website, macOS distribution must add:

- Apple Developer ID certificate storage in GitHub Actions secrets
- signed macOS app bundles
- signed DMG output
- Apple notarization
- a release checklist that verifies first-open behaviour on a clean macOS user account

Unsigned or ad-hoc signed artifacts are acceptable only for internal UAT.
