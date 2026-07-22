# System Pulse

System Pulse is a cross-platform desktop software project in founding-engineering setup.

The repository is the long-term home for a production-quality application that will eventually support macOS, Windows, and Linux from a shared architecture. The current macOS UAT app is a Tauri desktop build with real local system data.

## Codex Startup And Governance

Every Codex session for this repository must begin by reading `START_HERE.md` from the central [Codex Operating System](https://github.com/vanessanz78/codex-operating-system) repository before reading or changing project files.

Use the project-level [START_HERE.md](START_HERE.md) as a lightweight redirect only. The central Operating System governs document reading order, branch governance, engineering standards, architecture principles, milestone workflow, build verification, UAT requirements, cleanup procedures, and handoff requirements.

Do not duplicate Operating System documentation in this repository. Keep this repository focused on System Pulse product, architecture, implementation, and release knowledge.

## Vision

Keep your computer performing like the day you bought it.

System Pulse exists to become a trusted AI System Engineer for the computer: calm, local-first, lightweight, understandable, and able to help users diagnose, explain, and safely resolve problems without needing to become computer experts.

The Companion remains the quiet first surface. The AI System Engineer is the reasoning and recovery layer underneath it.

## Mission

Help knowledge workers confidently maintain a fast, healthy computer without needing technical expertise.

System Pulse should answer four questions quickly:

1. Am I okay?
2. What is happening?
3. Why does it matter?
4. What can I safely do about it?

The product should translate system signals into plain-English recovery plans, then execute only safe, real, user-approved care actions.

## Current Status

macOS UAT in progress: menu bar presence, quick check-in, Today, local collectors, PulseCore reasoning, first user-approved care actions, and the architecture path toward AI-assisted diagnosis and recovery.

The app now includes a Tauri/Rust backend, TypeScript frontend, local macOS collectors, and a PulseCore module that turns collector data into Today, Quick Check-in, and care-opportunity output.

Current GitHub desktop artifacts are internal UAT builds. Public macOS distribution requires Apple Developer ID signing and notarization so users can download, install, and double-click the app without a Terminal workaround.

Durable handoff notes:

- [Founding Engineering Handover](docs/00-Founding-Engineering-Handover.md)
- [Source Document Reading Notes](docs/19-Source-Document-Reading-Notes.md)
- [Founding Engineer Readiness Note](docs/18-Founding-Engineer-Readiness.md)
- [The Three Layers of PulseCore](docs/22-PulseCore-Layers.md)
- [Desktop Build And Download](docs/23-Desktop-Build-And-Download.md)
- [macOS Release Signing And Notarization](docs/34-macOS-Release-Signing-And-Notarization.md)
- [Decision Architecture](docs/33-Decision-Architecture.md)
- [Architect's Update 006: AI System Engineer](docs/41-Architects-Update-006-AI-System-Engineer.md)

## Repository Structure

```text
system-pulse/
|-- START_HERE.md
|-- README.md
|-- docs/
|-- design/
|-- assets/
|-- website/
|-- app/
|-- packages/
`-- .github/
```

### Top-Level Areas

| Path | Purpose |
| --- | --- |
| `docs/` | Founding source documents, read-through notes, implementation guardrails, and milestone notes. |
| `design/` | Future visual references and design-system source files. |
| `assets/` | Future shared static assets. |
| `website/` | Future public website or documentation website workspace. |
| `app/desktop/` | Tauri desktop app for the first macOS milestone. |
| `packages/` | Future shared libraries, domain packages, platform adapters, or tooling packages. |
| `.github/` | GitHub community standards, issue templates, discussion templates, and pull request templates. |

## Architecture Direction

The founding source documents specify:

- Tauri desktop shell.
- Rust core.
- TypeScript frontend.
- Local SQLite database.
- Platform-specific collectors.
- Local-first PulseCore.
- macOS and Windows first, Linux later once the core architecture is stable.

PulseCore follows a layered model: Observation, Diagnosis, Recovery Plan, and Safe Execution. Collectors observe, PulseCore reasons, the product experience protects the user's focus and momentum, and care actions run only after explicit user approval.

Desktop installers are built through GitHub Actions, not Replit. Replit is reserved for the future sales website and lightweight repository checks.

## Run The App On macOS

Prerequisites:

- Rust toolchain.
- Node.js and pnpm.
- macOS development tools required by Tauri.

Install dependencies and launch the app:

```bash
pnpm install
pnpm dev
```

The first milestone app reads local macOS memory, storage, processor, browser, and application signals, sends those snapshots to PulseCore, and renders the Companion and Today surfaces from PulseCore output.

## Development Philosophy

System Pulse should be built as a durable software product, not a prototype.

Core principles:

- Trust before convenience.
- Clarity before technical display.
- PulseCore interprets; UI displays.
- Collectors observe; they do not recommend.
- Local-first by default.
- One recommendation at a time.
- No hidden optimisation.
- No fake actions.
- No fear-based messaging.
- Low resource use is a product requirement.
- AI must not run at startup or in the background by default.
- Explain first, preview where possible, act only with approval.
- Experience always wins.

## Contribution Philosophy

Contributions should be small, reviewable, and tied to the founding documents.

Before implementation begins, contributions should focus on:

- clarifying source documents;
- recording open questions;
- proposing architecture decisions;
- improving repository standards;
- identifying risks, assumptions, and validation steps.

Feature work should follow the approved Version One scope summarized in [Founding Engineer Readiness Note](docs/18-Founding-Engineer-Readiness.md), the current sprint, and the AI System Engineer direction in [Architect's Update 006](docs/41-Architects-Update-006-AI-System-Engineer.md).

## Roadmap Summary

1. Foundation.
2. Heartbeat.
3. Understanding.
4. Recovery Planning.
5. Safe Care Actions.
6. Ask Pulse.
7. Workspace Intelligence.
8. Team Pulse.
9. Pulse Platform.

Version One is focused on Heartbeat, Understanding, and the first narrow Recovery Plan loop: the Companion, Today, System Score, basic PulseCore, one useful recommendation, local history, clear explanations, and only real user-approved care actions.

## License

Proprietary / all rights reserved for now. See `LICENSE`.