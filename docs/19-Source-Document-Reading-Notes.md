# Source Document Reading Notes

> Date read: 2026-06-29
> Scope: Full extracted text from the System Pulse Word documents plus both visual reference PNGs.
> Purpose: Durable engineering notes so future implementation starts from the founding documents, not from memory.

## Documents Read

- Product Bible.
- Experience Bible.
- North Star.
- Personality & Tone.
- Visual Language Bible.
- PulseCore Intelligence.
- Journey Library.
- Trust Model.
- Architecture Bible.
- Roadmap.
- Interaction Bible.
- PulseCore Specification.
- System Pulse Playbook.
- Release Bible Version One.
- PulseCore Reasoning Engine.
- Future Vision.
- Evolution Framework.

## North Star

System Pulse exists to keep the user's computer performing like the day they bought it.

The product must answer:

1. Is my computer healthy?
2. If not, why?
3. What is the single best thing I can do?

System Pulse must not become a technical dashboard, optimiser, fear-based utility, or engagement product.

## Product And Experience

The first emotion should be relief.

The default experience is Today, not a dashboard. The user should see The Heart, System Score, one recommendation, and enough plain-language context to return to work.

Technical metrics are advanced details only. Applications should be ranked by impact, not by raw memory. Recommendations must estimate benefit and show confidence.

Tone is calm, competent, friendly, reassuring, thoughtful, and quietly intelligent. It should never be dramatic, patronising, or technical by default.

## Trust Model

The Trust Model takes precedence over convenience.

Key rules:

- The computer belongs to the user.
- Local-first is the default.
- Permissions are explained before they are requested.
- Every recommendation is explainable.
- No fear marketing.
- Notifications are earned.
- Data minimisation is a product feature.
- Recommendations are never commands.
- Actions require consent.
- No hidden optimisation.

## Architecture

The source documents specify Tauri, Rust, TypeScript, SQLite, platform-specific collectors, and local-first PulseCore.

The application should be separated into:

- User Interface.
- PulseCore.
- System Collectors.
- Local Data Store.
- Optional Cloud Services.

Data flow:

Collectors produce normalized system snapshots. PulseCore interprets and reasons. The UI consumes PulseCore outputs. The UI never interprets raw metrics.

macOS and Windows are first-release targets. Linux follows once the core architecture is stable.

## PulseCore

PulseCore is a reasoning engine, not a monitoring engine.

Stages:

1. Observe.
2. Interpret.
3. Reason.
4. Prioritise.
5. Recommend.

PulseCore evaluates memory, processor, storage, applications, system, and power. It produces System Score, confidence, one recommendation, expected improvement, plain-language explanation, domain summaries, and advanced details.

Rules:

- Context before metrics.
- Patterns before snapshots.
- Prediction before diagnosis.
- Confidence before certainty.
- One recommendation before many.
- Silence is a feature.

Version One uses universal deterministic reasoning and deterministic language templates. Adaptive learning and cloud AI are future work.

## Version One Scope

Included:

- The Heart.
- Menu bar on macOS.
- System tray on Windows.
- Today.
- System Score.
- Basic PulseCore.
- Memory, CPU, storage, applications, battery, and system health.
- One recommendation.
- Expected improvement.
- Confidence.
- Plain-language explanation.
- Application insights.
- Simple local history.
- Weekly review.
- Minimal settings.
- Permissions flow.
- Accessibility.

Explicitly out of scope:

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

## Visual And Interaction Direction

The Heart is the relationship between the user and their computer.

It should breathe, not beat dramatically. One complete pulse is approximately two seconds. Motion is optional and must respect reduced-motion settings.

Colour has meaning:

- Green: healthy.
- Amber: attention recommended.
- Red: immediate action genuinely required.

The inspected visual references show:

- White Heart plus ring as the recommended menu bar concept.
- Calm score chip in the menu bar/system tray.
- Today, Insights, Applications, History, Weekly Review, Recommendation Detail, Settings, Permissions, and empty-state screen concepts.
- Design principles: one-glance understanding, calm and reassuring, actionable not overwhelming, information hierarchy, trust through transparency.

## Roadmap And Evolution

Milestone 0 is foundation. Milestone 1 is Heartbeat. Milestone 2 is Understanding. Later milestones are Learning, Protection, Workspace Intelligence, Team Pulse, and Pulse Platform.

Future Vision is inspiration, not commitment. Version Two should emerge from actual Version One usage, especially founder usage.

## Implementation Readiness

The next engineering step should be a minimal scaffold that proves the architecture without implementing product features too early.

Recommended first implementation milestone:

- Tauri app shell.
- Rust workspace boundaries.
- TypeScript frontend shell.
- PulseCore interface contracts.
- Collector interface contracts.
- SQLite decision record.
- No product UI beyond a placeholder health surface.

The goal is to validate package structure, local startup, and resource discipline before building the real Heart or Today screen.
