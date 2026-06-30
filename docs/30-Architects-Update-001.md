# Architect's Update 001

Date: 30 June 2026

## Purpose

This document records what we learned after the first real implementation and founder testing.

These learnings supersede earlier assumptions where they differ.

The Product Bible remains true.

The implementation strategy has evolved.

## The Biggest Discovery

System Pulse is not a system monitor.

System Pulse is a momentum manager.

Users do not care about RAM.

Users do not care about CPU.

Users do not care about renderer processes.

Users care about one thing.

Can I keep working?

Everything else exists only to answer that question.

## Health Is Not The Product

We originally believed the product was computer health.

Real-world testing showed something different.

The product is protecting momentum.

Health is merely one signal.

The user experience is the outcome.

## Experience Health

Experience Health is now considered a first-class PulseCore domain.

PulseCore should always prioritise how the computer feels over how the operating system measures itself.

Experience is more important than technical perfection.

## Flow Remaining

One of the strongest discoveries from founder testing.

Users naturally think in time.

Not percentages.

Not gigabytes.

Not CPU.

PulseCore should estimate remaining uninterrupted work time.

Examples:

- 4h 12m remaining.
- 52 minutes remaining.
- 8 minutes remaining.

This becomes the most meaningful metric inside System Pulse.

The menu bar score remains.

The dashboard communicates time.

## The Menu Bar Is The Product

The dashboard is not the primary interface.

The menu bar is.

Interaction hierarchy:

1. Menu Bar
2. Quick Check-in
3. Today
4. Care
5. Advanced

Most users should rarely open the dashboard.

The dashboard exists for investigation, not daily use.

## The Quick Check-in

Clicking the Heart should open a small summary.

The summary answers:

- How am I?
- How long have I got?
- Do I need to do anything?

If the answer is "No", the user closes the popup and returns to work.

This is the primary workflow.

## PulseCore Should Hide Complexity

PulseCore should perform deep reasoning.

The interface should expose almost none of it.

Never show:

- renderer processes,
- memory pressure,
- swap,
- WindowServer,

unless users explicitly request details.

PulseCore reasons.

Users simply receive the conclusion.

## Recommendations Become Care

Recommendations are evolving into Care.

Eventually this becomes Next Best Step.

Every recommendation should represent the lowest-disruption action that meaningfully extends the user's momentum.

## Preserve Active Work

PulseCore should understand active work.

Never recommend restarting an application currently performing important work if lower-impact alternatives exist.

Example lower-disruption sequence:

1. Close Safari.
2. Close Chrome.
3. Restart Finder.

Only then suggest restarting Codex.

This reasoning is central to the product.

## Timing Is The Product

PulseCore should optimise timing, not simply recommendations.

The ideal recommendation is:

- the right action,
- at the right moment,
- with the lowest interruption cost.

This is PulseCore's greatest competitive advantage.

## Interruption Threshold

System Pulse should interrupt users only when the cost of interruption is lower than the cost of remaining silent.

Silence is often the correct product decision.

## New PulseCore Principle

Protect Momentum.

Not Performance.

Performance exists only because it supports momentum.

Momentum is the product.

## Product Evolution

Future development should prioritise:

- Flow Remaining,
- lowest-disruption care,
- autonomous assistance,
- context awareness,
- decision timing.

Not additional technical metrics.

## Final Principle

System Pulse succeeds when users forget they are maintaining a computer.

Instead, they simply keep working.

That remains our goal.

From this point forward, every implementation decision should ask:

"Does this help the user keep working?"
