# Milestone 2: First Usable App

> Status: Implemented as source code.
> Date: 2026-06-29
> Scope: Smallest real macOS app with local system data.

## What Was Built

- Tauri desktop scaffold under `app/desktop`.
- Rust backend with a Tauri command named `get_today_pulse`.
- TypeScript/Vite frontend with a dark Today screen.
- The Heart using the approved white-heart and calm ring direction.
- PulseCore module that owns score, state, explanation, recommendation, confidence, and expected improvement.
- Real macOS memory collector.
- Real macOS storage collector.
- Real macOS top-applications-by-memory collector.

## Real Data

The app collects real local data on macOS:

- total memory;
- available memory;
- used memory;
- compressed memory;
- root storage total/available/used;
- top running applications grouped by memory impact.

The UI receives interpreted PulseCore output only. It does not score or interpret raw metrics.

## Placeholder Data

- System Score is a simple Version One placeholder formula owned by PulseCore.
- Confidence and expected improvement are deterministic placeholder values owned by PulseCore.
- Local history and SQLite are not implemented yet.
- The app is a desktop window first; menu bar/system tray behavior remains a follow-up milestone.

## Guardrails Preserved

- No cloud.
- No account.
- No hidden optimisation.
- No automatic app restarts.
- No destructive cleanup.
- Collectors do not recommend.
- UI does not interpret raw metrics.
- PulseCore owns recommendations.

## Run Command

```bash
pnpm install
pnpm dev
```

## Recommended Next Milestone

Add the macOS menu bar presence and lightweight refresh loop, then introduce SQLite-backed local history only after the live Today screen is stable.
