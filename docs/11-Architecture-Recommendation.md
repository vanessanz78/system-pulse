# Architecture Recommendation

| Field | Value |
| --- | --- |
| Status | Recommendation only |
| Owner | TBD |
| Last Updated | 2026-06-29 |
| Implementation State | Not started |

## Purpose

Recommend a long-term technical direction for System Pulse before application implementation begins.

This document is not an implementation plan. It should be reviewed and approved before any application code, UI, build tooling, or desktop packaging is created.

## Evaluation Priorities

The architecture should prioritize:

- cross-platform desktop support for macOS, Windows, and Linux;
- high performance;
- low memory usage;
- native look and feel where it materially affects trust and usability;
- long-term maintainability;
- simple deployment;
- automatic updates.

## Options Considered

### Flutter

Flutter supports native desktop builds for Windows, macOS, and Linux from a shared codebase.

Strengths:

- mature cross-platform UI framework;
- consistent rendering across platforms;
- strong tooling and documentation;
- good fit for highly custom interfaces.

Trade-offs:

- UI is rendered by Flutter rather than using each platform's native controls directly;
- native look and feel must be recreated intentionally;
- desktop deployment and update strategy require additional decisions;
- the project would be tied to Dart and Flutter-specific UI patterns.

### Tauri

Tauri uses a Rust application core with a system WebView frontend. It supports desktop applications across macOS, Windows, and Linux and provides a plugin ecosystem for desktop capabilities.

Strengths:

- strong fit for low memory and small application footprint goals;
- Rust is well suited to high-performance local system work;
- can use a focused web UI layer while keeping core logic native and strongly typed;
- supports desktop packaging and updater workflows;
- avoids bundling a full browser runtime in the way Electron does.

Trade-offs:

- the UI is still webview-based, so truly native controls require separate work;
- platform WebView differences must be tested carefully;
- Rust increases the skill requirement for contributors;
- native look and feel depends on disciplined design and OS-specific polish.

### Electron

Electron is a mature desktop application platform built around Chromium and Node.js.

Strengths:

- largest ecosystem among the options considered;
- fast path for web-heavy application development;
- strong packaging and update patterns;
- broad contributor familiarity.

Trade-offs:

- higher memory and disk footprint because each app ships a browser runtime;
- less aligned with the low memory priority;
- native feel typically requires extra work;
- long-term maintainability can suffer if app logic, UI, and platform code are not separated carefully.

### Native Swift/C#

A native split would likely mean Swift or AppKit/SwiftUI for macOS, C# with WinUI or Windows App SDK for Windows, and a separate Linux approach.

Strengths:

- best native look and feel;
- best access to platform-specific APIs;
- strong performance potential;
- aligns closely with each operating system's interface conventions.

Trade-offs:

- highest maintenance cost across three desktop targets;
- duplicated UI and platform logic;
- slower feature delivery;
- Linux would need a separate toolkit decision;
- release and update pipelines would diverge by platform.

## Recommendation

Recommend Tauri as the initial application architecture.

Proposed direction:

- Tauri desktop shell;
- Rust core for system integration, local data handling, and performance-sensitive work;
- minimal web frontend for the interface layer;
- shared packages for domain logic and cross-platform contracts;
- GitHub-based release workflow with signed artifacts and an update manifest when release work begins.

## Rationale

Tauri is the best fit for the stated priorities because it balances cross-platform reach, low memory goals, high-performance local code, and maintainable separation between desktop shell, core logic, and interface layer.

It is not the most native option. Native Swift/C# would win on platform fidelity, but it would create multiple product implementations before the product is even defined. Electron would be the fastest familiar web path, but its runtime footprint conflicts with the low memory goal. Flutter is a strong cross-platform option, but its custom-rendered UI is less aligned with native desktop feel unless the design system is built carefully around platform expectations.

## Suggested Repository Shape After Approval

Do not create these implementation folders yet. This is a proposed direction only.

```text
app/
  desktop/          # Tauri app shell after approval
packages/
  core/             # Shared Rust domain logic after approval
  contracts/        # Cross-platform schemas and interface contracts after approval
website/            # Public site or documentation site after approval
```

## Decisions Required Before Implementation

- Product scope and first milestone definition: TBD
- Required system permissions and privacy posture: TBD
- Local storage model: TBD
- Offline behavior: TBD
- Target OS versions: TBD
- Distribution channels: TBD
- Code signing approach: TBD
- Automatic update channel strategy: TBD
- Telemetry and diagnostics policy: TBD
- License: TBD

## Source Notes

- Tauri architecture and updater documentation: https://v2.tauri.app/concept/architecture/ and https://v2.tauri.app/plugin/updater/
- Flutter desktop documentation: https://docs.flutter.dev/platform-integration/desktop
- Electron updater documentation: https://electronjs.org/docs/latest/api/auto-updater
- Apple SwiftUI and AppKit documentation: https://developer.apple.com/documentation/swiftui and https://developer.apple.com/documentation/appkit
- Microsoft .NET MAUI and Windows App SDK documentation: https://learn.microsoft.com/en-us/dotnet/maui/ and https://learn.microsoft.com/en-us/windows/apps/windows-app-sdk/
