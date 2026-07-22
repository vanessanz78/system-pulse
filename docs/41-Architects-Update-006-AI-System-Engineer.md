# System Pulse

# Architect's Update 006

## AI System Engineer

Date: 23 July 2026

## Purpose

System Pulse should evolve from a passive health companion into an AI System Engineer: a calm, local-first assistant that diagnoses, explains, and safely helps fix computer problems on behalf of the user.

This does not replace the Companion direction. It completes it.

The Companion remains the quiet first surface. The AI System Engineer is the reasoning and recovery layer that appears when the user asks for help or when System Pulse has one genuinely useful action to offer.

## Product Correction

System Pulse is not Activity Monitor.

System Pulse is not a dashboard.

System Pulse is not a generic chatbot.

System Pulse is an experienced systems engineer built into the computer.

The product should answer:

1. What is happening?
2. Why is it happening?
3. What is the safest useful action?
4. What benefit should the user expect?

The user should never need to translate CPU, RAM, renderer processes, swap, cache, disk activity, or background jobs into a decision. PulseCore should do that translation.

## Core Example

Weak experience:

```text
CPU is high.
```

Desired experience:

```text
Chrome is running many renderer processes and Codex is actively indexing a workspace.
The least disruptive improvement is to suspend inactive browser tabs first.
Estimated benefit: recover memory and reduce processor pressure without interrupting active work.
```

## Product Model

The product now has four layers:

```text
Observation
  -> Diagnosis
  -> Recovery Plan
  -> Safe Execution
```

Observation collects local signals.

Diagnosis correlates those signals and explains them in plain English.

Recovery Plan chooses the least disruptive useful care path.

Safe Execution performs only user-approved actions that are real, reversible where possible, and never destructive without explicit confirmation.

## Surface Model

### Companion

The Companion remains the default surface.

It answers quickly:

- Am I okay?
- How long have I got?
- Anything I should know?
- Need more?

The Companion should stay small, calm, and glanceable.

It should not become a chat surface or a dashboard.

### Today

Today remains the user's calm plan.

It may show a Recovery Plan when action is useful.

The plan should be ordered from least disruption to most disruption.

### Ask Pulse

Add an assistant entry point only where it helps the user ask for help naturally.

Accepted prompt examples:

- My Mac feels slow.
- Free up space.
- Why is Chrome using so much memory?
- Can I close anything safely?
- Optimise my development environment.
- Why are my fans running?
- My battery drains too quickly.

Ask Pulse should not overwhelm the first view. It can live behind a small input, side panel, or detail view, but the Companion must remain the first breath.

## Recovery Plans

Recovery Plans replace static recommendations when the app has enough evidence.

A plan should include:

- Estimated time.
- Expected benefit.
- Interruption level.
- Recommended actions.
- Preview.
- Run.
- Explain.
- Ignore or Later.

A Recovery Plan should appear only when there is something real and useful to do.

If no safe action is implemented, System Pulse should explain the situation calmly and avoid showing a fake action button.

## One-Click Care Scope

System Pulse may eventually support safe, approved actions across these areas:

- Storage: empty Trash, remove old installers, clear obsolete caches, remove temporary build folders, clean package-manager caches, identify large recoverable files.
- Memory: restart Finder, suspend inactive applications, restart browser renderer processes, recover inactive memory where the operating system safely allows it.
- Browser: suspend inactive tabs, detect duplicate tabs, offer tab grouping, clear browser cache with consent.
- Developer: clean npm, pnpm, yarn, build, and temporary project caches; detect abandoned repositories; remove obsolete generated artifacts.
- Battery: identify high-impact background apps and suggest power-saving changes.
- Network: identify heavy transfers, stalled requests, or unusual background traffic.

These are not all Version One commitments. They are the durable direction for the AI System Engineer capability.

## Safety Rules

System Pulse must earn trust before it earns automation.

Every action must follow these rules:

1. Preview before running whenever practical.
2. Explain why the action was suggested.
3. Show expected benefit.
4. Show expected interruption.
5. Require confirmation before deleting or changing user data.
6. Never delete personal documents automatically.
7. Prefer reversible actions.
8. Never optimise hidden behaviour without the user's consent.
9. Never interrupt protected active work such as Codex while it is actively helping the user.
10. Never recommend restarting the Mac before smaller recovery paths have been considered.

## Low-Compute Rule

AI must not run at startup.

AI must not run in the background by default.

AI should run only when:

- The user asks Ask Pulse a question.
- The user opens Today and requests an explanation.
- The app has a documented, approved, user-visible recovery-plan reason.

System Pulse must remain lightweight while idle.

If cloud AI is ever introduced, it must be explicit, privacy-aware, and documented before implementation. Local-first reasoning remains the default product posture.

## Architecture Direction

PulseCore should own diagnosis and recovery reasoning.

Collectors observe facts only.

The UI displays calm decisions and approved actions.

The assistant should consume structured PulseCore output rather than rediscovering system state from scratch.

The durable contract should move toward:

```text
SystemSnapshot
  -> PulseDiagnosis
  -> RecoveryPlan
  -> CareAction
  -> ExecutionResult
```

Each CareAction should include:

- id
- title
- reason
- expectedBenefit
- interruptionLevel
- riskLevel
- requiresConfirmation
- reversible
- implemented
- previewAvailable
- executor

## Implementation Sequence

Do not start by adding a broad chatbot.

Recommended sequence:

1. Define the Recovery Plan and CareAction contracts.
2. Convert the current recommendation panel to use Recovery Plan language.
3. Add one fully wired, low-risk care action with Preview, Explain, Run, and Later.
4. Add Ask Pulse as a thin natural-language entry point over existing structured diagnoses.
5. Add more care actions only after the safety pattern is proven.
6. Add AI generation only after local deterministic diagnosis has a stable contract.

## Version One Slice

The first useful slice should be narrow:

- Detect the main pressure source.
- Explain it in plain English.
- Recommend the least disruptive next action.
- Offer one real, safe care action if implemented.
- Hide the action if it is not implemented.

Success is not how much the app knows.

Success is whether Vanessa can keep working with less stress.

## UAT For This Direction

1. Open Companion and confirm it still feels like a calm Apple-style check-in, not a chatbot or dashboard.
2. Open Today during a real pressure moment and confirm the right panel reads like a Recovery Plan.
3. Confirm the plan explains what is happening, why it matters, the smallest useful action, and expected benefit.
4. Confirm no fake action is visible.
5. Confirm any visible action can be previewed or explained before running.
6. Confirm System Pulse does not run AI or heavy diagnosis while idle.
7. Confirm protected active work is not interrupted by default.

## Decision

Adopt AI System Engineer as the core long-term vision for System Pulse.

Keep the Companion as the quiet first product surface.

Build automation through trust: explain first, preview where possible, act only with approval, and preserve user momentum above technical completeness.
