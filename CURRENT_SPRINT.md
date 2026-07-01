# System Pulse

# CURRENT_SPRINT

Sprint: Companion, Not Software

Status: Active

## Source Priority

Read this file before the Product Bible, README, or any older process document when deciding what to change next.

Then read `docs/37-Feeling-Bible.md`.

Then read `docs/38-Candle-Pulse-Visual-Guide.md`.

## Sprint Goal

Transform System Pulse from a dashboard into a trusted companion.

Every screen should help the user feel calm, in control, reassured, and ready to keep creating.

## Mental Model

Stop designing System Pulse like software. Start designing it like a trusted companion that quietly checks in.

Design System Pulse as if Apple were redesigning the Battery menu, not Activity Monitor.

System Pulse is glanceable, calm, reassuring, time-based, and action-only-when-needed.

Product One is the Companion.

Product Two is Today's Plan.

The Companion is the Battery menu.

Today is an inbox.

Only things requiring attention should remain visible. Everything else should disappear.

## Primary Product Goal

System Pulse answers four questions:

1. Am I okay?
2. How long have I got?
3. Anything I should know?
4. Need more?

Nothing else should compete with these questions.

## Current Problems

- The app keeps drifting back toward categories instead of reassurance.
- Today still risks becoming a dashboard instead of an inbox.
- Applications, memory, storage, browser, and flow can become feature cards instead of tiny status signals.
- Technical terminology leaks into the experience.
- Large action buttons make calm maintenance feel urgent.
- Disabled or fake actions would damage trust immediately.

## Design Direction

The Companion is the product.

Today's Plan is the inbox.

The Companion should be understandable in under three seconds.

Today's Plan exists only when the user chooses to investigate.

## Visual North Star

The Candle Pulse visual guide is now the durable style reference:

- `docs/38-Candle-Pulse-Visual-Guide.md`

The Companion should visually feel like a compact Apple menu popover:

- Dark rounded panel with a small top notch
- Large green circular score ring with the heart inside it
- Reassurance copy beside the score, not a dashboard heading
- Estimated uninterrupted work time in green
- Quiet "At a Glance" list with Applications, Storage, Battery, and Memory
- One calm Open Today button

Use the Candle Pulse guide for visual language, mood, density, and hierarchy.

Do not use the expanded reference image as permission to turn Today back into Activity Monitor.

## Companion Requirements

Display only:

- Heart
- Pulse Score
- Greeting
- Calm state sentence
- Estimated uninterrupted work time
- Quiet at-a-glance checks for Applications, Memory, Storage, and Battery
- Open Today

Nothing else.

No restart button, next best step, local check label, live label, version number, or diagnostic explanation in the Companion.

## Today's Plan Requirements

Today is not a dashboard.

Today should become one calm plan:

- Greeting
- "You're good for" time remaining
- Things worth knowing
- Do I need to do anything?

No Flow card, Browser card, Memory card, Storage card, Applications card, or Next Best Step card.

If the user clicks a specific item, then show detail for that item only.

Every section should either reassure, recommend, or disappear.

Only show an action when it is genuinely implemented. If it is not wired yet, hide it or mark it as coming soon.
