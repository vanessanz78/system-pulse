# System Pulse

# CURRENT_SPRINT

Sprint: Decision Support, Not Diagnostics

Status: Active

## Source Priority

Read this file before the Product Bible, README, or any older process document when deciding what to change next.

## Sprint Goal

Transform System Pulse from a dashboard into a decision-support companion.

Every screen should help the user make the next right decision about their Mac.

## Mental Model

Design System Pulse as if Apple were redesigning the Battery menu, not Activity Monitor.

System Pulse is glanceable, calm, reassuring, time-based, and action-only-when-needed.

The popup is the Battery menu.

Today is an inbox.

Only things requiring attention should remain visible. Everything else should disappear.

## Primary Product Goal

System Pulse answers three questions:

1. Can I keep working?
2. Approximately how long do I have before slowdown becomes likely?
3. What is the smallest action that buys me more uninterrupted work?

Nothing else should compete with these questions.

## Current Problems

- The popup contains too much competing copy.
- Today repeats information already shown in the popup.
- Today still risks becoming a dashboard instead of an inbox.
- Applications present information before the user asks for detail.
- PulseCore reasoning is still too exposed.
- Technical terminology leaks into the experience.
- Large buttons make calm maintenance feel urgent.

## Design Direction

The popup is the product.

The dashboard is the explanation.

The popup should be understandable in under five seconds.

The dashboard exists only when the user chooses to investigate.

## Popup Requirements

Display only:

- Heart
- Pulse Score
- Estimated uninterrupted work time
- One direct answer to "Can I keep working?"
- Quiet at-a-glance checks for Applications, Memory, and Storage
- One recommended action only if needed
- Open Today

Nothing else.

## Dashboard Requirements

Today is not a dashboard.

Today should become one calm summary page:

- Greeting
- Current shape
- Estimated uninterrupted work time
- Things worth knowing
- Do I need to do anything?

No Flow card, Browser card, Memory card, Storage card, Applications card, or Next Best Step card.

If the user clicks a specific item, then show detail for that item only.

Every section should either reassure, recommend, or disappear.
