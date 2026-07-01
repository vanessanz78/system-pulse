# System Pulse

# Architect's Update 004

## Battery, Not Activity Monitor

Date: 1 July 2026

## Purpose

The product model is now clear:

Design System Pulse as if Apple were redesigning the Battery menu, not Activity Monitor.

System Pulse should feel glanceable, calm, reassuring, time-based, and action-only-when-needed.

## Correction

The previous direction still risked building the wrong product.

The intended product is:

```text
Heart
How am I?
Can I keep working?
How long have I got?
One thing, if anything
Done
```

The product must avoid becoming:

```text
Computer
Analysis
Diagnosis
Recommendations
Actions
```

That second model is why the experience feels heavy even when layout, copy, and colour improve.

## Popup Model

The popup is the Battery menu.

It answers:

1. Am I okay?
2. How long have I got?
3. Anything I should know?
4. Need more?

It should show:

- Heart
- Pulse Score
- Direct answer
- Estimated uninterrupted work time
- Quiet at-a-glance checks
- Open Today

One recommended action appears only if needed.

## Today Model

Today is not a dashboard.

Today is an inbox.

It should show one simple summary page:

```text
Good afternoon, Vanessa.

You're in good shape.

Estimated uninterrupted work time

4 hours.

Things worth knowing

- Chrome has been running for 20 hours.
- Codex is currently active.
- Storage has 37 GB free.

Do I need to do anything?

Restart Chrome
Later
Ignore
```

The dashboard should not show Flow, Browser, Memory, Storage, Applications, or Next Best Step cards.

If the user clicks a specific item, the detail page should show only that item.

## Detail Model

Detail belongs behind intent.

When the user clicks Chrome, they should see Chrome only:

```text
Chrome

Running

20 hours.

Why it matters.

Estimated benefit

+35 minutes

Restart now
Later
Ignore
```

## Button Model

Care actions should feel like Apple controls, not warnings.

Use quiet language:

- Restart now
- Later
- Ignore

Avoid shouting through size, weight, or urgency unless the system is genuinely critical.

## Product Promise

System Pulse is almost invisible until it can help.

It reassures first.

It recommends only when needed.

It explains only when asked.
