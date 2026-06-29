# Architecture

| Field | Value |
| --- | --- |
| Status | In progress |
| Owner | TBD |
| Last Updated | 2026-06-30 |
| Decision State | Revision 001 captured |

## Purpose

Architecture decisions for System Pulse. This file starts as a compact working architecture note; the richer source documents should remain the deeper product reference.

## Sections To Complete

- Runtime architecture: Tauri shell, Rust backend, TypeScript frontend.
- Platform architecture: platform collectors feed a shared snapshot model.
- PulseCore: owns interpretation, scoring, confidence, recommendation and expected improvement.
- UI: displays interpreted PulseCore output only.
- Collectors: observe raw/local system facts only and never recommend.
- Storage strategy: local SQLite is planned, not implemented.
- Security model: local-first, no account, no hidden optimisation, no destructive action without consent.
- Release architecture: TBD.

## PulseCore Domains

Current implemented domains:

- Memory Health
- Storage Health
- Application Impact
- Browser Health
- Renderer Health
- WindowServer Health on macOS

The Milestone 3 Revision adds Browser, Renderer and WindowServer reasoning because real founder usage showed that a computer can feel slow while high-level memory, CPU and storage still look acceptable.

## Real-World Learning Loop

New observations should be recorded under `docs/learnings/`.

The learning loop is:

Observe -> Use -> Learn -> Refine

Each learning should describe the real-world event, the product learning, the PulseCore reasoning change, and what remains unsupported or future research.

## Open Questions

- How should browser tab count be collected without invasive permissions?
- Which macOS APIs can safely measure desktop responsiveness without adding overhead?
- When should WindowServer observations become a user-facing explanation versus an internal confidence signal?

## Decision Log

- 2026-06-30: Real-World Learning 001 added Browser Health, Renderer Health and WindowServer Health as PulseCore reasoning domains.
