# Recovery Recommendation Engine

## Purpose

The Recovery Recommendation Engine chooses the single best action to protect flow.

It must not recommend actions because an application is large. It should recommend the action that offers the most Focus Time improvement with the least interruption, highest safety, and highest trust.

## Contract

The first contract is `RecoveryCandidate`.

It includes:

- `domain`
- `actionKind`
- `target`
- `expectedGainMinutes`
- `estimatedInterruptionSeconds`
- `confidence`
- `safetyLevel`
- `requiresConfirmation`
- `canAutomate`
- `sessionPreservationRisk`
- `reason`
- `trustNotes`

Post-action verification is represented by `ActionResult`.

It includes:

- `actionKind`
- `target`
- `startedAt`
- `completedAt`
- `success`
- `interruptionSeconds`
- `beforePrediction`
- `afterPrediction`
- `actualGainMinutes`
- `errors`
- `userCancelled`

## Ranking Model

Candidates should be ranked by:

```text
expectedGain * confidence
- interruptionCost
- safetyPenalty
- trustPenalty
- sessionPreservationPenalty
```

The current implementation introduces the candidate contract and conservative initial ranking while preserving existing UI behavior.

## Safety Rules

Hard rules:

- Never restart Codex while it is active work.
- Never restart or quit unknown applications.
- Never recommend restarting the Mac first.
- Never show a fake action.
- Never hide the interruption cost.
- Never automate destructive cleanup.
- Always require confirmation for application quit or restart until trust rules are more mature.

## Action Types

Current safe automation surface:

- restart known browsers where macOS can quit and reopen the app
- quit Safari
- restart Finder
- open Storage Settings

Guidance-only candidates remain valid when no safe automation exists.

## Browser Recovery

Browser recovery should be treated as useful but not risk-free.

Known risks:

- private windows may not restore
- unsaved form input may be lost
- uploads or downloads may be interrupted
- some authenticated tabs may need refresh or sign-in
- browser restoration behavior differs by browser and user settings

The current contract records `sessionPreservationRisk` so future UI can explain this clearly before a one-click recovery.

## Confidence

Initial confidence is conservative because there is no historical post-action validation yet.

Future confidence should increase when:

- the same browser or app repeatedly responds well to the same recovery
- measured Focus Time improves after the action
- the user accepts the action without later undoing or ignoring it

Future confidence should decrease when:

- the user cancels or ignores the action
- post-action Focus Time does not improve
- the application is protected active work
- the target has unknown session preservation behavior

## One-Click Recovery

One-click recovery should mean one clear product action, not one unsafe click.

Recommended future flow:

1. Rank candidates.
2. Choose one primary candidate.
3. Check safety and session risk.
4. Require confirmation when quitting or restarting an app.
5. Execute only if allowed.
6. Collect a fresh prediction.
7. Store an `ActionResult`.
8. Compare expected gain with actual gain.

## Trust Model

Trust is part of the ranking.

System Pulse should prefer:

- no interruption over unnecessary action
- guidance over unsafe automation
- browser restart over Mac restart
- protected active work over reclaiming resources
- truthful uncertainty over false confidence

The recovery engine exists to protect uninterrupted work, not to chase cleaner metrics.
