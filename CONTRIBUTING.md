# Contributing

System Pulse is in foundation setup.

No application features, user interface, platform implementation, build system, or runtime code should be added until the relevant product and architecture decisions are approved.

## Codex Operating System

This repository is governed by the central [Codex Operating System](https://github.com/vanessanz78/codex-operating-system).

Before contributing with Codex, read the central Operating System `START_HERE.md` first, then follow its instructions for document reading order, branch governance, engineering standards, architecture principles, milestone workflow, build verification, UAT, cleanup, and handoff.

Do not copy the Operating System into this repository. If a workflow rule belongs to every Vanessa project, update the central Operating System instead. This repository should keep only System Pulse-specific product, architecture, implementation, and release guidance.

## Contribution Scope

Current acceptable contributions:

- improve repository standards;
- clarify placeholder documents without making product decisions;
- record open questions;
- propose architecture decisions for review;
- improve issue, pull request, and discussion templates.

Out of scope until approved:

- application code;
- UI implementation;
- desktop packaging;
- telemetry;
- product features;
- production infrastructure.

## Pull Request Expectations

Pull requests should be small and reviewable.

Each pull request should explain:

- what changed;
- why it changed;
- what is intentionally out of scope;
- how the change was checked;
- what follow-up decisions remain.

## Documentation Standards

Do not treat placeholders as decisions. If a product, design, architecture, brand, or roadmap point is not approved, mark it as `TBD` or add it to an open questions section.

## Local Development

Avoid generating large local artifacts unless they are required for the approved task. Generated caches and build outputs should not be committed.
