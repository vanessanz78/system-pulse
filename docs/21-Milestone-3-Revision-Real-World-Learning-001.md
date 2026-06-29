# Milestone 3 Revision: Real-World Learning 001

> Status: Implemented as a PulseCore revision.
> Date: 2026-06-30
> Scope: Browser, renderer and WindowServer reasoning.

## What Changed

This milestone revision captures the first founder-driven System Pulse learning:

Traditional health signals can look acceptable while the computer still feels slow.

PulseCore now begins reasoning about user experience by adding:

- Browser Health;
- Renderer Health;
- WindowServer Health on macOS.

## Implementation

The macOS collector now observes:

- browser process groups for Chrome, Safari, Edge and Firefox where process names are visible;
- browser renderer/helper process counts where process names expose renderer-like roles;
- largest browser renderer memory;
- browser process uptime where available from process elapsed time;
- WindowServer memory;
- WindowServer CPU.

PulseCore now:

- scores Browser Health separately from general Memory Health;
- scores Renderer Health separately from general Application Health;
- scores WindowServer Health separately on macOS;
- weights the System Score toward user-perceived responsiveness;
- prioritises browser-renderer recommendations when they are the strongest explanation;
- explains why the browser is likely affecting responsiveness in plain English.

## Guardrails Preserved

- Collectors observe only.
- Collectors do not recommend.
- UI does not interpret raw metrics.
- PulseCore owns scoring and recommendations.
- No cloud.
- No account.
- No automatic optimisation.
- No hidden restarts.
- No destructive cleanup.

## Current Limitations

- Tab count is not implemented.
- Browser memory growth over time is not implemented.
- WindowServer growth over time is not implemented.
- Screenshot latency and desktop animation smoothness are research items only.
- Local history is still required before PulseCore can compare current behaviour with previous sessions.

## Success Criteria

When sluggishness is primarily caused by browser renderer growth, System Pulse should identify the browser as the likely cause, explain why in plain English, and recommend one clear action.

## Recommended Next Step

Finish macOS UAT for the current Tauri app, then add menu bar presence and manual refresh while preserving the new PulseCore boundaries.
