# Polish Sprint 001

| Field | Value |
| --- | --- |
| Status | Applied |
| Source | Lyra design review shared by Vanessa |
| Captured | 2026-06-30 |
| Decision State | Product polish direction |

## Purpose

This sprint refines the first usable System Pulse app without adding new features.

The goal is to make Today feel less like a dashboard and more like a calm daily check-in with the user's computer.

## Feedback Summary

The first app milestone successfully captured the calm, trustworthy direction:

- It does not feel like Activity Monitor.
- It does not feel like CleanMyMac.
- The white Heart and green halo direction works.
- The hierarchy is close: Heart, score, health state, recommendation.
- The recommendation language lowers stress.

The next pass should reduce dashboard language and increase human explanation.

## Polish Goals

- Reduce visible raw metrics.
- Increase plain-English interpretation.
- Make Today conversational.
- Replace app classifications such as High and Low with meaningful context.
- Make the Heart smaller and softer.
- Let the score carry more emotional weight.
- Keep advanced metrics available, but secondary.

## Applied Changes

- Today now opens with a time-aware greeting and an experience line.
- The health statement uses "feels" language to reflect the Experience Layer.
- The Heart is smaller, with a softer breathing halo.
- The score is visually stronger.
- Memory and storage cards lead with plain-English experience statements.
- Raw domain values moved behind a Details disclosure.
- Application rows now show PulseCore context instead of High, Medium, and Low classifications.
- The recommendation area now reads as a daily check-in rather than a report.

## Guardrails Preserved

- Collectors still collect only local system data.
- Collectors do not recommend.
- PulseCore owns scoring, labels, explanations, and recommendations.
- The UI renders PulseCore output and does not interpret raw metrics.
- No cloud.
- No account.
- No automatic optimisation.
- No destructive cleanup.

## Deferred

- Personal greeting by name is deferred until System Pulse has a local, privacy-respecting way to know the user's preferred name.
- Historical "more than usual" application context is deferred until local history exists.
- Full source-original document archival remains tracked separately in `docs/24-Source-Archive-Status.md`.
