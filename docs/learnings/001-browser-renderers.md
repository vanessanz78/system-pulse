# Learning 001: Browser Renderers Can Make a Healthy System Feel Slow

## Status

Implemented as a first PulseCore revision on 2026-06-30.

## Observation

During normal founder development work:

- Codex became slow.
- The Mac became sluggish.
- Screenshots took much longer than usual.
- Codex eventually terminated with `SIGKILL`.
- Memory pressure appeared healthy.
- CPU was mostly idle.
- Disk space was healthy.

Investigation showed Chrome had accumulated many renderer/helper processes consuming several gigabytes of memory. The system looked healthy through traditional high-level metrics, but the user experience was degraded.

## Product Learning

System Pulse cannot only ask whether memory, CPU and storage look healthy.

PulseCore must ask:

Why does the computer feel slow?

Browser behaviour is a first-class health domain because browsers can dominate perceived responsiveness through renderer growth, helper processes and long sessions.

## PulseCore Response

PulseCore now receives browser observations from the macOS collector:

- browser process count;
- browser total memory;
- browser renderer count;
- largest renderer memory;
- browser process uptime where available.

PulseCore scores Browser Health and Renderer Health separately from general memory health. Browser-renderer pressure can now lower the System Score and produce the primary recommendation even when broad memory and storage signals look acceptable.

## Recommendation Direction

Preferred wording:

Chrome appears to be the largest contributor to your computer's current resource usage. Most of this comes from browser renderer processes that can accumulate during long browsing sessions. Restarting Chrome would likely improve responsiveness.

## Not Implemented Yet

- Real tab count.
- Browser-specific session history across launches.
- Renderer growth trend over time.
- Recovery measurement after restarting the browser.

These require local history, browser integration, or permissions that should be researched before implementation.
