# System Pulse

# CURRENT_SPRINT

Sprint: Decision Support, Not Diagnostics

Status: Active

## Source Priority

Read this file before the Product Bible, README, or any older process document when deciding what to change next.

## Sprint Goal

Transform System Pulse from a dashboard into a decision-support companion.

Every screen should help the user make the next right decision about their Mac.

## Primary Product Goal

System Pulse answers three questions:

1. Can I keep working?
2. Approximately how long do I have before slowdown becomes likely?
3. What is the smallest action that buys me more uninterrupted work?

Nothing else should compete with these questions.

## Current Problems

- The popup contains too much competing copy.
- Today repeats information already shown in the popup.
- Applications present information instead of actions.
- PulseCore reasoning is still too exposed.
- Technical terminology leaks into the experience.

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
- One sentence answering "Can I keep working?"
- One recommended action only if needed
- Open Today

Nothing else.

## Dashboard Requirements

Do not repeat the popup.

Instead, answer:

- Why?
- What should I do?
- How much uninterrupted work will this action buy me?

Every section should either reassure, recommend, or disappear.
