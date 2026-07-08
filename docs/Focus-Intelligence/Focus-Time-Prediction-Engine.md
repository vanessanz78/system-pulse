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

The Sprint 2 implementation populates this contract from the Focus Time Prediction Engine without changing the approved UI or menu bar behavior.

## Prediction Model

The current implementation replaces score-derived Focus Time buckets with a prediction model:

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
4. Blend immediate risk with sustained pressure, rising pressure, and volatility penalties.
5. Convert the resulting risk into remaining minutes and state.

Focus Time is now the authoritative source for Estimated Uninterrupted Work Time. Legacy `flowRemaining` fields remain populated for compatibility, but they mirror `FocusPrediction.remainingMinutes`.

## Sprint 2 Implementation Notes

The engine uses existing collectors only. No new background collectors or persistent monitors were introduced.

Current risk normalization uses:

- score-derived signal risk as a baseline
- memory availability, swap, and renderer memory ratios
- browser renderer count, largest renderer size, CPU, and uptime
- processor reserve
- active application pressure and protected work flags
- WindowServer memory and CPU
- disk throughput
- storage availability

Current adjustment layers:

- sustained pressure: applied when multiple contributors are under pressure, browser sessions are long-running under load, swap is building, or disk/desktop pressure is active
- rising pressure: estimated from concurrent moderate/severe contributors
- volatility: applied when one signal is sharply worse than the rest
- weak-signal caps: severe contributors cap the prediction at red/orange/yellow thresholds
- protected work: active protected work still reduces predicted smooth time, but it is not treated as an actionable recovery target

## Confidence

Confidence is calculated from:

- collector freshness: the current snapshot is marked fresh at collection time
- collector completeness: missing WindowServer data lowers confidence
- signal stability: sustained pressure and volatility reduce confidence
- conflicting indicators: a wide spread between healthy and unhealthy signals lowers confidence

Confidence remains deliberately capped because the product does not yet store history or validate predictions after recovery actions.

## Deviations

The architecture originally called for continuous recent-history trend detection. Sprint 2 does not introduce a persistent history store, so trend and volatility detection use current-snapshot proxies instead:

- concurrent moderate/severe contributors imply rising pressure
- long-running browser sessions under load imply sustained browser pressure
- swap, disk, and WindowServer pressure imply lower stability
- wide signal spread implies volatile conditions

This keeps the implementation within the sprint constraint to use existing collectors only. A future sprint should add a lightweight sample history so rising trends can be measured directly rather than inferred.

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
