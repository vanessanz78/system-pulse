# Founding Engineer Readiness Note

> Status: Milestone handoff after reading the founding source documents.
> Date: 2026-06-29
> Implementation state: No production application code has been written.

## What Was Completed

- Read the founding engineering handover.
- Read the full extracted text of all Word documents in the System Pulse archive.
- Inspected both visual reference images.
- Preserved the repository as documentation-only; no app code, UI, framework scaffold, dependency install, or build output was created.
- Added durable GitHub notes summarizing the architecture direction, Version One scope, implementation boundaries, and next milestone.

## Architecture Direction From Source Documents

The source documents specify:

- Tauri desktop shell.
- Rust core.
- TypeScript frontend.
- Local SQLite database.
- Platform-specific collectors.
- Local-first PulseCore.
- macOS and Windows first, Linux later once the core architecture is stable.
- No mandatory account, cloud connection, subscription, or cloud AI in Version One.

## Non-Negotiable Engineering Boundaries

- The UI never interprets raw metrics.
- Collectors never make recommendations.
- PulseCore alone produces the System Score, recommendation, confidence, expected improvement, and plain-language explanation.
- Recommendations are one-at-a-time.
- Permissions must be explained before they are requested.
- Actions require consent.
- No hidden optimisation.
- No fear-based messaging.
- Local-first is the default.
- System Pulse must not become resource-heavy itself.

## Version One Scope

Version One includes:

- The Heart.
- Menu bar on macOS.
- System tray on Windows.
- Today screen.
- System Score.
- Basic PulseCore.
- Memory, CPU, storage, application, battery, and system health.
- One primary recommendation.
- Expected improvement.
- Confidence.
- Plain-language explanation.
- Simple local history.
- Weekly review.
- Minimal settings.
- Accessibility support including reduced motion, keyboard navigation, screen readers, high contrast, and colour-independent communication.

Version One explicitly excludes:

- Cloud sync.
- Accounts.
- Team management.
- Predictive AI.
- Adaptive learning.
- Automatic optimisation.
- Cleaning tools.
- Browser extensions.
- Public API.
- Plugin marketplace.
- Mobile application.

## Visual Reference Notes

The attached visual references were inspected locally.

Observed direction:

- Recommended menu bar treatment is a white Heart with a green ring and visible score.
- Health states use green for healthy, amber for attention, and red only for immediate action.
- Motion should be slow, subtle, and purposeful.
- Version One screen guide includes Today, Insights, Applications, History, Weekly Review, Recommendation Detail, Settings, Permissions, menu bar/system tray, and empty states.

The PNG files were prepared locally for import, but local git push authentication was unavailable in this Codex environment. The design references should be committed in a later authenticated GitHub sync.

## Questions Requiring Founder Input

1. Confirm that the next milestone should be a minimal Tauri/Rust/TypeScript/SQLite scaffold rather than more documentation refinement.
2. Confirm whether Version One should start macOS-only for the first installable build while preserving Windows architecture from day one.
3. Confirm the preferred license before code is added.
4. Confirm whether the Heart visual reference should be treated as directionally approved or concept-only.
5. Confirm the first real collector target: memory, CPU, storage, or running applications.

## Recommended Next Milestone

Create the technical scaffold only:

- Tauri workspace under `app/desktop`.
- Rust workspace boundaries for PulseCore and collectors.
- TypeScript frontend shell with no product UI beyond a placeholder health surface.
- SQLite dependency decision recorded but not overbuilt.
- Platform collector interfaces only.
- No feature-complete screens until the scaffold proves packaging, local startup, and resource footprint.

The milestone should end with an installable development build only if it can be produced without violating the lightweight local-machine constraint.
