# Focus Time Prediction Engine

## Purpose

The Focus Time Prediction Engine estimates how long the user can continue working smoothly before the Mac is likely to interrupt flow.

Focus Time Remaining is a prediction, not a countdown timer. It should adjust as system conditions change and should become the core product signal for System Pulse.

## Contract

The first contract is `FocusPrediction`.

It includes:

- `remainingMinutes`
- `state`: `green`, `yellow`, `orange`, or `red`
- `confidence`
- `primaryReducer`
- `contributors`
- `lastUpdated`
- `staleness`
- `menuBarState`

Each contributor is represented by `FocusContributor`:

- `domain`
- `label`
- `state`
- `risk`
- `impactMinutes`
- `reason`
- `supportingMetrics`
- `protectedWork`
- `actionAvailable`

## Inputs

Initial inputs come from the existing local macOS collectors:

- Memory: available memory, used memory, compressed memory, swap used.
- Processor: user CPU, system CPU, idle reserve.
- Browser: browser memory, browser CPU, process count, renderer count, largest renderer, uptime.
- Applications: user-opened non-browser applications, memory, CPU, protected work.
- Desktop responsiveness: WindowServer memory and CPU.
- Disk activity: current read/write pressure.
- Storage: free space and used space.

## State Model

Focus state is determined from predicted remaining minutes:

- Green: 61 or more minutes.
- Yellow: 31 to 60 minutes.
- Orange: 15 to 30 minutes.
- Red: 14 minutes or less.

The current implementation introduces this contract without changing the approved UI or menu bar behavior.

## Prediction Model

The current implementation still derives Focus Time from the existing PulseCore score buckets. This is intentional for the first contract step.

The next implementation step should replace score-derived buckets with a real prediction model:

1. Normalize each signal into a risk value from `0.0` to `1.0`.
2. Apply product weights:
   - Memory pressure: 25%.
   - Browser and renderer pressure: 20%.
   - Processor reserve: 20%.
   - Active application pressure: 15%.
   - Desktop responsiveness: 8%.
   - Disk activity: 7%.
   - Storage space: 5%.
3. Apply caps for severe pressure so a healthy average cannot hide an urgent reducer.
4. Blend immediate risk with trend and volatility once history exists.
5. Convert the resulting risk into remaining minutes and state.

## Confidence

Initial confidence is calculated from the strength and spread of current signals. It is intentionally conservative because the product does not yet store history or validate predictions after recovery actions.

Future confidence should include:

- sample freshness
- collector completeness
- signal stability
- recent trend reliability
- whether similar recommendations previously improved Focus Time
- whether the user ignored or deferred similar recommendations

## Refresh Frequency

The low-compute rule still applies.

Recommended future behavior:

- Visible Today or Companion: refresh about every 60 seconds.
- Green menu bar state: lightweight refresh every 5 to 10 minutes.
- Yellow or orange: refresh every 2 to 3 minutes.
- Red: refresh every 60 seconds while active or until dismissed.
- Expensive disk checks should run only while visible or when pressure is suspected.

## Failure Modes

Known failure modes:

- short CPU spikes that resolve before the next sample
- stale predictions after sleep or wake
- browser session state not captured
- protected active work mistaken for safe recovery work
- disk pressure caused by temporary indexing or syncing
- external drives or mounted volumes changing disk readings
- unknown applications that cannot be safely automated

## Future Learning

Future learning should compare predictions before and after recovery actions.

The engine should record:

- prediction before action
- action selected
- expected gain
- interruption time
- prediction after action
- actual gain
- whether the user cancelled, deferred, or ignored the action

This turns System Pulse from a static score model into an Operations Manager that learns which actions protect uninterrupted work.
