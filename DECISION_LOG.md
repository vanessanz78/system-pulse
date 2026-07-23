# Decision Log

## 2026-07-23 - PulseCore Mission Engine

System Pulse now treats Storage Recovery as the first registered Pulse Mission rather than a one-off feature.

Decisions:

- Introduce `PulseMission`, `MissionAction`, `MissionResult`, Mission lifecycle, Mission Registry, and Mission Planner as reusable contracts.
- Register Storage Recovery as the reference implementation for future missions.
- Keep Today visually unchanged while moving it to consume generic mission objects from the registry.
- Keep Ask Pulse as a routing abstraction over structured mission output, not a chatbot.
- Record mission start, completion, cancellation, deferral, duration, and verification locally only.

Implications:

- Browser, Developer, Battery, Network, Security, Applications, Updates, and Health missions should extend the Mission Engine instead of creating bespoke UI orchestration.
- CareActions own execution and verification. PulseCore and the UI should not execute system changes directly.
- Future missions must use the standard lifecycle states documented in `docs/42-Mission-Engine.md`.

## 2026-07-23 - AI System Engineer core vision

System Pulse should evolve from a passive monitoring dashboard and calm health companion into an AI System Engineer: a local-first assistant that diagnoses, explains, plans recovery, and executes safe care actions only with user approval.

Decisions:

- Adopt `docs/41-Architects-Update-006-AI-System-Engineer.md` as the durable architecture direction for the next product capability layer.
- Keep the Companion as the quiet first surface. The AI System Engineer is the reasoning and recovery layer underneath it, not a replacement for the calm Companion experience.
- Ask Pulse should be introduced only as a natural-language entry point over structured PulseCore diagnoses, not as a broad generic chatbot.
- Recovery Plans should answer what is happening, why it matters, the smallest useful action, expected benefit, and expected interruption.
- Visible actions must be real, safe, previewable where practical, and user-approved. Fake or unwired actions remain hidden.
- AI must not run at startup or in the background by default. It should run only after user intent or an approved, documented, user-visible recovery-plan trigger.
- The implementation sequence should begin with durable Recovery Plan and CareAction contracts before adding broader chat or automation.

Implications:

- PulseCore should own diagnosis and recovery reasoning.
- Collectors should continue to observe facts only.
- The UI should display calm decisions and approved actions.
- Future care actions must preserve the low-compute standard and must never delete or change user data without explicit confirmation.

## 2026-07-05 - Low compute architecture standard

System Pulse now follows Engineering Standard 001: a deployed app with zero active users should consume close to zero compute.

Decisions:

- The desktop app shell may start, but the main window now starts hidden.
- System Pulse must not collect a local system snapshot on app launch.
- The 60 second refresh loop may run only while the app window is visible and useful.
- The refresh loop must stop when the window is hidden, blurred, or unloaded.
- The website remains static: no analytics, AI, realtime services, dashboards, or admin bundles on the landing page.
- Future features must follow `docs/Engineering-Standards/001-Low-Compute-Architecture.md`.

Implementation and audit:

- `docs/40-Low-Compute-Audit-And-Reduction.md`
- `app/desktop/src/main.ts`
- `app/desktop/src-tauri/tauri.conf.json`

## 2026-07-02 - Data clarity visual pass

Vanessa approved the current Candle Pulse-style visual direction and asked for the next refinement to match the attached Today view reference:

- Companion score should be a clean green circle, not a heart outline.
- Today should keep the four visual status cards: Applications, Memory, Browser, Storage.
- Each Today card should show a compact metric footer with useful numbers.
- Memory should be framed as RAM, because it affects sluggishness and app switching.
- Storage should be framed as disk space, because it affects updates, caches, and reliability.
- Things worth knowing should show compact right-side metrics.
- Recommended care should become a short Needs attention action list: Application, Memory, Browser, Storage.

A local implementation commit was created but could not be pushed from this Mac because local GitHub auth is not configured and the connector upload path was not reliable for large files.

Local checkpoint, if still present:

- Path: /Users/vanessa/Documents/Codex/2026-07-01/git-5/work/system-pulse
- Commit: 52c98c0235a14b071c85d517870c5ab6f22f7bde
- Message: Clarify Today metrics and care actions
- Parent main at time of work: 4c130bdc1bd0efeff0625a2d932fe11d5b74395f

Changed locally in that checkpoint:

- CURRENT_SPRINT.md
- package.json
- app/desktop/package.json
- app/desktop/src-tauri/tauri.conf.json
- app/desktop/src-tauri/src/models.rs
- app/desktop/src-tauri/src/pulse_core.rs
- app/desktop/src/main.ts
- app/desktop/src/styles.css

Verification completed locally:

- git diff --check passed
- JSON parse passed for package.json, app/desktop/package.json, app/desktop/src-tauri/tauri.conf.json
- Non-ASCII scan passed for edited source/docs files

Verification not completed locally:

- cargo fmt/check/build could not run because Cargo is not installed on this Mac.
- The downloadable macOS artifact was not produced because the implementation commit was not pushed.
