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
- Large green circular score ring with the score inside it, no heart shape
- Reassurance copy beside the score, not a dashboard heading
- Estimated uninterrupted work time in green
- Quiet "At a Glance" list with Applications, Storage, Battery, and Memory
- One calm Open Today button

Use the Candle Pulse guide for visual language, mood, density, and hierarchy.

Do not use the expanded reference image as permission to turn Today back into Activity Monitor.

## Companion Requirements

Display only:

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

## Current Data Clarity Pass

Date: 2 July 2026

Vanessa confirmed the overall visual direction is working and asked for the next refinement:

- The Companion score should be a clean green circle like the expanded Today score, not a heart outline.
- The four Today cards should show the useful underlying numbers at the bottom of each card.
- Memory should be framed as RAM, because this is what makes applications feel sluggish.
- Storage should be framed as disk space, because it affects updates, caches, and reliability.
- The Things worth knowing list should pair each note with the relevant measure where possible.
- Recommended care should read as short "Needs attention" action rows, not a wordy question.

UAT for the next build:

1. Open the Companion and confirm the score is a green circle with no heart shape.
2. Open Today and confirm Applications, Memory, Browser, and Storage each have a metric footer.
3. Confirm Memory shows RAM available/used and Storage shows disk free/used with percentages.
4. Confirm Things worth knowing shows compact right-hand metrics.
5. Confirm Recommended care is headed "Needs attention" and uses short action rows.

## Current Intelligence Pass

Date: 2 July 2026

Vanessa clarified that System Pulse is not trying to explain Activity Monitor. It is trying to prevent the Mac from becoming sluggish while she is in flow.

System Pulse should treat the Pulse score like a fuel gauge plus early warning light:

- OK: the Mac is not close to reserve; keep working.
- Later: finish the current thought, then tidy the main pressure source.
- Care: the Mac is on reserve and flow may be interrupted soon.

The intelligence should prioritize signals that affect smooth work right now:

- CPU reserve, especially low idle CPU.
- RAM pressure, especially swap and compressed memory.
- Browser renderer load, because Chrome/Safari/Edge/Firefox can become many heavy processes.
- Disk activity, because reads and writes can make apps feel slow even when storage space is not full.
- Storage space, because low space affects updates, caches, and reliability.

Applications should exclude browsers. Browser pressure belongs only in Browser. Applications should show the next highest non-browser app pressure.

UAT for the next build:

1. Open Today while Activity Monitor is visible and confirm Browser is not repeated under Applications.
2. Confirm Memory reflects swap pressure when Activity Monitor shows significant swap used.
3. Confirm Storage reads as used/total, while also noting active disk activity when the disk is busy.
4. Confirm yellow or red states only appear when the Mac is close to reserve, not for manageable background load.
5. Confirm the recommended action is the least disruptive useful care step, not a generic diagnosis.

## Current Storage Wording Fix

Date: 2 July 2026

Vanessa caught a mismatch where Storage could say "Good" while also saying free disk space was lower than ideal, even when only a small percentage of the disk was used.

Storage copy must separate two signals:

- Disk space: how much of the drive is used.
- Disk activity: whether reads/writes are busy right now.

If disk space has room, the card should say that clearly. Disk activity may reduce smoothness, but it should not make the app claim free space is low.

UAT for the next build:

1. Confirm a low used percentage, such as 7% used, says disk space has room.
2. Confirm "Free disk space is lower than ideal" appears only when the used/free storage percentage actually supports it.
3. Confirm disk-busy language appears only when disk activity is the reason for attention.

## Current CPU Pressure Truth Fix

Date: 2 July 2026

Vanessa caught a case where System Pulse said the Mac was in a good place to focus while opening Chrome still felt sluggish.

The screenshot showed the actual reason:

- Codex was using meaningful CPU.
- Desktop responsiveness was also using meaningful CPU.
- Browser itself was not the current pressure source.

System Pulse must not treat this as "No action needed." If the Mac feels slow and active application or desktop responsiveness CPU is high, the score should drop into Later and show the calm care path.

Implementation direction:

- Keep Browser separate from Applications.
- Let high application CPU or desktop responsiveness CPU reduce the overall flow score, even if RAM and browser look fine.
- If Codex is the pressure source, show it as protected active work to review, not something to restart mid-task.
- Keep the wording calm: "finish what you're doing first," not panic.

UAT for the next build:

1. Open Today while Codex or Desktop responsiveness is using noticeable CPU and confirm the score does not stay high green.
2. Confirm the right panel shows a review/care row instead of "No action needed."
3. Confirm Browser can still say Good when Chrome itself is not the pressure source.
4. Confirm Codex is not offered as a restart action while it is actively working.
