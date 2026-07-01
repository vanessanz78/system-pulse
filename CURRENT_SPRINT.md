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

## Current Implementation Pass

Date: 1 July 2026

This pass implements the Candle Pulse visual guide in the actual desktop UI, not only in documentation.

- The Companion moves away from a chart-like score circle and uses a glowing heart score mark.
- The Companion keeps the Candle Pulse structure: warm greeting, time remaining, quiet at-a-glance rows, and one Open Today button.
- The at-a-glance rows now use small visual status symbols for Applications, Memory, Browser, and Storage.
- Today opens as a dark glass window with a title bar, score and greeting hero, time as the primary readout, things worth knowing, one care panel, and a calm reassurance strip.
- The pulse shapes stay whole and clean: no pie-chart wedge, cut-out slice, or conic progress shadow.

UAT for the next build:

1. Open the menu bar Companion and confirm it visually resembles the Candle Pulse menu bar reference.
2. Confirm the heart/score mark is not a bland circular chart and has no pie-slice shadow.
3. Click Open Today and confirm the expanded view feels like a calm glass window, not a plain report.
4. Confirm only one care suggestion appears when action is useful.

## Current Visual Correction

Date: 1 July 2026

Vanessa clarified that the Candle Pulse visual guide should be copied more literally for the expanded Today view.

Current instruction:

- Keep the Companion calm and glanceable.
- Today should visually include the four small filled status cards from the right-hand Candle Pulse guide: Applications, Memory, Browser, and Storage.
- Those cards are visual status tiles, not a return to a heavy diagnostic dashboard.
- The recommendation panel should match the guide structure: small recommended label, one care action, estimated benefit, and quiet secondary action.
- Prefer matching the supplied visual reference first, then refine product simplification from that baseline.

UAT for the corrected build:

1. Open Today and confirm the row of four small status cards appears under the score/time hero.
2. Confirm the Browser card can show "Needs attention" while the other cards show Good.
3. Confirm the recommendation panel visually resembles the guide, including the orange primary action and quiet Later button.
4. Confirm the menu bar Companion includes the one-suggestion row above Open Today when there is a useful suggestion.
