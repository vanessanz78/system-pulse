# Desktop Build And Download

| Field | Value |
| --- | --- |
| Status | Initial build pipeline |
| Owner | TBD |
| Last Updated | 2026-07-01 |
| Decision State | Development artifacts first, public releases require signed distribution |

## Plain-English Model

System Pulse is a desktop application, not a website.

The app source code lives in GitHub. GitHub is the source of truth.

Tauri turns the source code into real desktop installers:

- macOS: `.app` and `.dmg` style bundles.
- Windows: installer packages such as `.msi` or `.exe`.
- Linux: packages such as `.AppImage`, `.deb`, or similar.

The sales website is separate. It will eventually link to these released installers.

## Current Build Flow

The current flow is:

1. Codex helps write the source code.
2. The source code is saved to GitHub.
3. GitHub Actions runs native desktop builds in the cloud.
4. GitHub stores the build output as downloadable workflow artifacts.
5. Vanessa downloads the macOS artifact and tests it on her Mac.

This avoids using Replit as the desktop build machine.

## Why Replit Is Not The App Build Environment

Replit is useful for:

- future sales website work;
- reading repository files;
- lightweight TypeScript checks;
- documentation review.

Replit is not the right place to verify the macOS desktop app because System Pulse needs native macOS behavior, including system collectors and future menu bar behavior.

## Development Artifacts Versus Public Releases

The first GitHub builds are development artifacts.

They are for internal testing only and may be unsigned or ad-hoc signed. macOS may warn before opening them. That is not acceptable for a customer-facing product download.

Public releases must include:

- code signing;
- notarization for macOS;
- stable download links;
- release notes;
- clear version numbers;
- update strategy.

## macOS Public Release Gate

A Mac build is customer-ready only when it is produced by the manual `macOS Signed Release` workflow and verified as signed and notarized.

The customer-ready requirement is simple: a user must be able to download the Mac build, drag it into Applications, and double-click it without running a Terminal command.

See [macOS Release Signing And Notarization](34-macOS-Release-Signing-And-Notarization.md) for the required Apple Developer credentials, GitHub Actions secrets, and release checklist.

## Current Workflow

The internal UAT workflow is defined in `.github/workflows/desktop-build.yml`.

It builds:

- macOS;
- Windows;
- Linux.

The workflow can run manually from GitHub Actions and also runs when desktop app source files change on `main`.

The customer-ready macOS release workflow is defined in `.github/workflows/macos-signed-release.yml`. It is manual-only and requires Apple Developer ID signing and notarization secrets before it can produce a public Mac artifact.

## Current Limitations

- The standard macOS artifact is not yet a polished public release.
- Apple Developer ID signing and notarization secrets are not configured yet.
- Automatic updates are not configured.
- The website does not yet link to release downloads.
- The full source-original document library still needs a separate import if those original files should live in GitHub.

## Next Product Step

Continue improving the macOS app first, while preparing the release account infrastructure required for customer-ready distribution.

The next app milestone should focus on:

- customer-ready signing and notarization setup;
- menu bar reliability;
- click-to-open Today;
- clearer loading and error states;
- preserving PulseCore as the only recommendation engine.