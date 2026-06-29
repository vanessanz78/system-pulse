# System Pulse

System Pulse is a cross-platform desktop software project in founding-engineering setup.

The repository is the long-term home for a production-quality application that will eventually support macOS, Windows, and Linux from a shared architecture. The founding product, experience, trust, architecture, and release materials have now been read and summarized into durable GitHub notes. Production application code has not started.

## Vision

Keep your computer performing like the day you bought it.

System Pulse exists to become a trusted computer health companion: calm, local-first, lightweight, and understandable without requiring users to become computer experts.

## Mission

Help knowledge workers confidently maintain a fast, healthy computer without needing technical expertise.

System Pulse should answer three questions quickly:

1. Is my computer healthy?
2. Why?
3. What should I do?

## Current Status

Founding-document read-through complete.

No application code, UI implementation, framework scaffold, build output, or product feature has been created yet.

Durable handoff notes:

- [Founding Engineering Handover](docs/00-Founding-Engineering-Handover.md)
- [Source Document Reading Notes](docs/19-Source-Document-Reading-Notes.md)
- [Founding Engineer Readiness Note](docs/18-Founding-Engineer-Readiness.md)

## Repository Structure

```text
system-pulse/
├── README.md
├── docs/
├── design/
├── assets/
├── website/
├── app/
├── packages/
└── .github/
```

### Top-Level Areas

| Path | Purpose |
| --- | --- |
| `docs/` | Founding source documents, read-through notes, implementation guardrails, and milestone notes. |
| `design/` | Future visual references and design-system source files. |
| `assets/` | Future shared static assets. |
| `website/` | Future public website or documentation website workspace. |
| `app/` | Future cross-platform desktop application workspace. |
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
- No fear-based messaging.
- Low resource use is a product requirement.

## Contribution Philosophy

Contributions should be small, reviewable, and tied to the founding documents.

Before implementation begins, contributions should focus on:

- clarifying source documents;
- recording open questions;
- proposing architecture decisions;
- improving repository standards;
- identifying risks, assumptions, and validation steps.

Feature work should follow the approved Version One scope summarized in [Founding Engineer Readiness Note](docs/18-Founding-Engineer-Readiness.md).

## Roadmap Summary

1. Foundation.
2. Heartbeat.
3. Understanding.
4. Learning.
5. Protection.
6. Workspace Intelligence.
7. Team Pulse.
8. Pulse Platform.

Version One is focused on Heartbeat and Understanding: The Heart, Today, System Score, basic PulseCore, one recommendation, local history, weekly review, and clear explanations.

## License

License terms have not been selected yet. See `LICENSE`.
