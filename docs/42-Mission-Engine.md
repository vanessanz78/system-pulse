# System Pulse
# Mission Engine

Date: 23 July 2026

## Purpose

The Mission Engine turns System Pulse care work into one reusable architecture.

Storage Recovery is the reference implementation. Browser, Developer, Battery, Network, Security, Applications, Updates, and Health missions should plug into the same engine later instead of creating separate UI and execution paths.

System Pulse should follow this shape:

```text
Collectors
  -> PulseCore Diagnosis
  -> Mission Planner
  -> Mission Engine
  -> Care Actions
  -> Execution
  -> Verification
```

Collectors observe facts only.

PulseCore diagnoses and chooses the smallest useful care path.

The Mission Engine orchestrates lifecycle, ranking, previews, execution, and result publishing.

CareActions perform the approved work and verify the result.

## PulseMission

Every mission uses the same product contract:

```ts
interface PulseMission {
  id: string
  category: string
  title: string
  summary: string
  confidence: string
  status: MissionLifecycle
  priority: number
  estimatedBenefit: string
  expectedBenefit: string
  expectedInterruption: string
  estimatedDuration: string
  diagnosis: string
  recoveryPlan: string
  actions: CareAction[]
}
```

The UI consumes `PulseMission` objects. It must not contain a hard-coded list of future mission types.

## Lifecycle

Every mission must use only these states:

```text
Observed
Diagnosed
Ready
Previewed
Approved
Running
Verifying
Completed
Deferred
Unavailable
Failed
```

Missions may not invent additional lifecycle states. This keeps Today, Companion, Ask Pulse, telemetry, and future mission views aligned.

## CareAction

Every future action should implement this shape:

```ts
interface CareAction {
  id: string
  title: string
  description: string
  confidence: string
  interruption: string
  preview(): MissionPreview
  explain(): MissionExplanation
  estimate(): MissionEstimate
  execute(): MissionResult
  verify(): MissionVerification
}
```

CareActions own execution. PulseCore and the Mission Engine should never delete files, close apps, clear caches, restart processes, or change settings directly.

## MissionResult

Results are standardised:

```ts
interface MissionResult {
  completed: boolean
  skipped: boolean
  failed: boolean
  recoveredSpace: string
  duration: string
  verification: string
}
```

Storage includes storage-specific measured fields such as free space before and after, but future missions should preserve the shared result fields.

## Registry

Missions register with the engine:

```ts
MissionRegistry.register(StorageMission)
```

The registry loads missions dynamically and publishes:

- `topMission`
- `otherOpportunities`

Today reads the top mission from the registry. Future mission types should not require a new Today layout.

## Planner

The Mission Planner ranks missions using:

- estimated benefit
- confidence
- user disruption
- estimated duration
- safety
- current system context

The first implementation ranks Storage Recovery by estimated recoverable space, confidence, interruption, and mission priority. Future planners can add richer context while keeping the same `PulseMission` output.

## Ask Pulse Routing

Ask Pulse should not begin as a chatbot.

The first abstraction is routing:

- storage questions route to Storage Mission
- otherwise Ask Pulse starts with Today's top mission
- if no mission is ready, Ask Pulse answers from the local Today health picture

This keeps natural-language entry points grounded in PulseCore output instead of rediscovering system state.

## Telemetry

Telemetry is local only.

System Pulse may record:

- mission started
- mission completed
- mission cancelled
- mission deferred
- execution duration
- verification result

No telemetry is sent anywhere.

## Extension Rules

Future missions must:

1. Use the standard `PulseMission` contract.
2. Register through `MissionRegistry`.
3. Keep collectors observation-only.
4. Keep execution inside CareActions.
5. Provide Preview, Explain, Run, and Verify before showing a real action.
6. Use plain English.
7. Avoid fake buttons and simulated results.
8. Protect active work and personal data.
9. Fit Today as one calm recommendation, not a dashboard.

## Sprint 2 Decision

Storage Mission is no longer special.

It is the first registered mission and the reference implementation for the Mission Engine.
