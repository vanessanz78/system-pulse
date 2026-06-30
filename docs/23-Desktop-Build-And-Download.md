# Desktop Build And Download

| Field | Value |
| --- | --- |
| Status | Initial build pipeline |
| Owner | TBD |
| Last Updated | 2026-06-30 |
| Decision State | Development artifacts first, public releases later |

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

They are for internal testing only and may be unsigned. macOS may warn before opening them.

Public releases will come later and should include:

- code signing;
- notarization for macOS;
- stable download links;
- release notes;
- clear version numbers;
- update strategy.

## Current Workflow

The workflow is defined in `.github/workflows/desktop-build.yml`.

It builds:

- macOS;
- Windows;
- Linux.

The workflow can run manually from GitHub Actions and also runs when desktop app source files change on `main`.

## Current Limitations

- The macOS artifact is not yet a polished public release.
- Code signing and notarization are not configured.
- Automatic updates are not configured.
- The website does not yet link to release downloads.
- The full source-original document library still needs a separate import if those original files should live in GitHub.

## Next Product Step

Continue improving the macOS app first.

The next app milestone should focus on:

- menu bar presence;
- click-to-open Today;
- manual refresh;
- clearer loading and error states;
- preserving PulseCore as the only recommendation engine.
