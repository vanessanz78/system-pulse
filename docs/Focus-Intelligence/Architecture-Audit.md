# Focus Intelligence Architecture Audit

## Objective

System Pulse is moving from system monitoring to Focus Intelligence.

The product mission is:

> How long can I continue working smoothly before my Mac interrupts my flow, and what is the smallest action I can take to stay in flow?

This audit compares the current implementation with the required Focus Intelligence architecture.

## Current Implementation

The current app has:

- local macOS collectors for CPU, memory, swap, storage, disk activity, applications, browsers, renderers, and WindowServer
- PulseCore scoring for domains and an overall system score
- an estimated uninterrupted work time label
- Today and Companion surfaces using the approved visual layout
- four primary diagnostic cards: Applications, Memory, Processor, Browser
- browser separation from application pressure
- System Pulse self-measurement guards
- protected Codex work handling
- limited wired recovery actions
- refresh only while visible or focused

## Already Aligned

System Pulse already aligns with the Operations Manager direction in these areas:

- The window starts hidden.
- The app does not collect a snapshot on launch.
- Refresh loops stop when hidden or blurred.
- Browser pressure is separated from Applications.
- Applications exclude many system internals.
- Codex can be treated as protected active work.
- The Today layout already includes the approved four diagnostic cards.
- Recovery UI already avoids fake buttons in many cases.

## Needs Refinement

The current architecture still depends on score-first thinking.

Needs refinement:

- Focus Time is derived from score buckets, not a prediction model.
- There is no confidence model.
- There is no contributor contract explaining why Focus Time is dropping.
- There is no ranked recovery candidate model.
- Expected gains are fixed strings.
- Browser session preservation is not modeled.
- Menu bar state is still score-driven.
- There is no notification threshold model.
- There is no post-action verification loop.
- There is no historical learning store.

## Contract Layer Added

This foundation step adds first-class contracts for:

- `FocusPrediction`
- `FocusContributor`
- `RecoveryCandidate`
- `MenuBarState`
- `ActionResult`

These contracts are serialized through the existing Today pulse response but are not yet used to redesign the UI, change menu bar behavior, or trigger notifications.

## Required Implementation Path

Recommended order:

1. Keep the new contracts stable and review them.
2. Replace score-derived Focus Time buckets with a true prediction model.
3. Make contributors the source of truth for why Focus Time is decreasing.
4. Replace text-only recommendation selection with ranked recovery candidates.
5. Add post-action verification using `ActionResult`.
6. Add historical learning after the verification loop exists.
7. Change menu bar behavior using `MenuBarState`.
8. Add notification behavior only after menu bar states are trustworthy.

## Non-Goals For This Step

This step does not:

- redesign Today
- redesign Companion
- change menu bar visuals
- add notifications
- change website behavior
- create a DMG
- add browser restart behavior
- add fake actions

## Product Guardrail

Every future intelligence change should answer:

1. Am I okay?
2. How long can I continue working?
3. What is reducing Focus Time?
4. What is the single best thing I can do?
5. What improvement will I get?

If a screen or engine output does not answer those questions, it is drifting back toward Activity Monitor.
