# Product Reset

## Stop Showing PulseCore

Date: 30 June 2026

## Purpose

This document records a product correction after founder testing.

The UI should not be a window into PulseCore.

The UI should be a window into the user's next decision.

## The Problem

The current interface exposes too much of PulseCore's reasoning.

The user does not need to see PulseCore thinking.

The user needs the outcome.

PulseCore should perform more reasoning so the interface can display less information.

## The Correct Question

Do not ask:

What information should we show?

Ask:

What decision is the user trying to make?

## The Only Three Questions

When the user clicks the Heart, the Quick Check-in should answer only:

1. Can I keep working?
2. Approximately how long before my computer starts getting in my way?
3. Do I need to do anything right now?

Nothing else belongs in the Quick Check-in.

## Delete The Noise

Do not expose these in the default experience:

- Working Hard
- Worth Watching
- Experience
- Momentum
- Confidence
- Expected Pulse
- Renderer counts
- Technical reasoning

These are PulseCore internals.

They may exist under an explicit details or advanced view only when they directly explain a recommendation.

## Quick Check-in

The popup should take approximately five seconds to read.

Example:

```text
Heart 84

Estimated uninterrupted work time
1h 50m

You're still working comfortably.

No action needed right now.

Applications OK
Storage OK
Memory OK

Open Today
```

That is enough.

The user should be able to close it and continue working.

## Dashboard Purpose

The dashboard exists only because the user has chosen to investigate further.

It should answer:

- Why?
- What is my next best step?

It should not repeat information already shown in the Quick Check-in.

## Every Card Must Answer So What

If a card cannot immediately help the user decide what to do, remove it.

## PulseCore Thinks

PulseCore can reason about:

- WindowServer
- Renderer counts
- Memory pressure
- Swap
- CPU
- Browser sessions
- Application uptime

The user should rarely see these.

Only show them if they directly explain a recommendation.

## Time Is The Product

The most valuable metric is estimated uninterrupted work time.

Every recommendation should answer:

How much more uninterrupted work will this buy me?

Examples:

- Restart Chrome: +35 minutes
- Close Safari: +12 minutes
- Restart Finder: +5 minutes

Gigabytes are not the product.

Time is.

## Lowest Disruption

Never recommend the largest optimisation by default.

Recommend the action that preserves active work.

If restarting Chrome buys enough time, recommend Chrome.

Do not recommend restarting the Mac unless that is the lowest-disruption useful action.

## The Test

Before displaying any piece of information, ask:

Will this help the user decide what to do?

If the answer is no, do not display it.

PulseCore already understands it.

The user does not need to.

## Final Principle

System Pulse should feel almost empty because PulseCore is carrying the complexity.

The more intelligent PulseCore becomes, the simpler the interface should become.
