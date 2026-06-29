# Learning 002: WindowServer Can Explain Perceived Sluggishness On macOS

## Status

Partially implemented as a first PulseCore signal on 2026-06-30.

## Observation

During the same founder usage session, desktop responsiveness degraded while high-level memory, CPU and storage signals still looked acceptable.

WindowServer memory usage had grown significantly.

This matters because WindowServer behaviour often correlates with symptoms users actually feel:

- slow screenshots;
- laggy window movement;
- Mission Control delays;
- delayed desktop interactions.

## Product Learning

System Pulse should reason about perceived responsiveness, not just hardware capacity.

WindowServer is not an ordinary application from the user's perspective. On macOS it is a dedicated PulseCore signal because it can explain desktop sluggishness even when the rest of the system looks fine.

## PulseCore Response

The macOS collector now observes WindowServer memory and CPU usage when available.

PulseCore now creates a WindowServer Health interpretation and can recommend a planned desktop restart if WindowServer is the clearest signal.

## Guardrails

- No automatic restart.
- No hidden optimisation.
- No panic wording.
- No red-text alarm language.
- Recommendation only, with user control.

## Not Implemented Yet

- Screenshot latency.
- Window animation smoothness.
- Mission Control responsiveness.
- WindowServer growth over time.

These are future research items. They should not be invented until there is a supported, reliable signal.
