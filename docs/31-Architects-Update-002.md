# Architect's Update 002

## Simplify Ruthlessly

Date: 30 June 2026

## Purpose

This Architect's Update records the most important usability discovery made during founder testing.

The implementation has become too focused on exposing PulseCore's reasoning.

Users do not need to understand PulseCore.

PulseCore exists so users do not have to think about technical complexity.

From this point forward, every design decision should remove cognitive load rather than add it.

## The Biggest Discovery

We have been designing from the inside out.

The founder thinks from the outside in.

PulseCore understands the computer.

Users simply want to know:

Can I keep working?

Everything else is secondary.

## The Product Is Not A Dashboard

The primary product is not the dashboard.

The primary product is the menu bar Heart.

Interaction hierarchy is now:

1. The Heart
2. Quick Check-in
3. Today
4. Care
5. Advanced

Most users should spend less than ten seconds interacting with System Pulse.

## The Quick Check-in

The Quick Check-in is now the primary interface.

When clicking the Heart, users should immediately know:

- How am I?
- How much uninterrupted time do I have left?
- Do I need to do anything?

Nothing else should compete for attention.

## The Dashboard Changes Purpose

The dashboard no longer exists to provide an overview.

The popup already provides the overview.

The dashboard exists to answer:

Why?

and

What is the best next step?

The dashboard is a diagnosis.

Not another summary.

## Simplify Ruthlessly

PulseCore should think more.

Users should think less.

Every piece of information must justify its existence.

If removing information makes the experience clearer, remove it.

## Remove Internal Language

The following concepts are internal PulseCore terminology.

Do not expose them in the default experience.

- Renderer
- Memory Pressure
- Swap
- WindowServer
- Confidence
- Expected Pulse
- Working Hard
- Experience Layer
- Momentum Layer

These may exist under Advanced.

Not in the default experience.

## Speak In Human Language

Replace technical labels with decisions.

Instead of:

Working Hard

Use:

- No action needed.
- Worth watching.
- Recommended today.
- Recommended now.

These are meaningful.

They lead naturally to action.

## Time Is More Valuable Than Percentages

Founder testing revealed that users naturally think in time.

Not percentages.

Not gigabytes.

Not CPU usage.

The primary metric inside System Pulse should become:

Estimated uninterrupted work time.

This should eventually evolve into:

Estimated time until you are likely to notice slower performance.

Time is the most meaningful measure of momentum.

## Context Before Numbers

Raw numbers rarely help.

Example:

3.1 GB

has almost no meaning without context.

Instead explain:

Codex is currently using more memory than your other applications.

This is expected while actively developing software.

Only expose raw values under Details.

## Applications Become Care Opportunities

Applications should no longer be presented as passive information.

Each application should answer:

Do I need to do anything?

If not:

Say nothing beyond reassurance.

If yes:

Provide one clear Suggested Care action.

Every application becomes an opportunity to extend momentum.

## Lowest Disruption Wins

PulseCore should recommend the lowest-disruption action that meaningfully extends uninterrupted work.

Example:

Rather than:

Restart Codex

PulseCore may instead recommend:

- Close Safari
- Restart Chrome
- Restart Finder
- Delay action

The best recommendation is not always the biggest improvement.

It is the one that preserves active work.

## Preserve Active Work

PulseCore must understand active work.

Never recommend restarting an application currently performing important work if safer alternatives exist.

Example:

Codex is actively generating.

Recommendation:

Wait until the task completes.

Then perform care.

Protecting work always comes before optimisation.

## Interruption Threshold

System Pulse should interrupt users only when silence becomes more harmful than interruption.

Until then:

Observe quietly.

Learn.

Wait.

Timing is part of the product.

## Information Hierarchy

Every interface should answer only one question.

Menu Bar:

How am I?

Quick Check-in:

Can I keep working?

Today:

Why not?

Care:

What should I do?

Advanced:

Show me the technical details.

Never mix these levels together.

## Score Interpretation

The score should communicate reassurance.

Not urgency.

Suggested interpretation:

- 95-100: Excellent
- 80-94: Working well
- 60-79: Worth watching
- 40-59: Plan some care
- 20-39: Recommended today
- 0-19: Act soon

A score of 84 should never make users anxious.

It should communicate:

You're still working well.

## The New Product Goal

The product no longer exists to explain computers.

It exists to answer one question.

Can I keep working?

Everything else supports that answer.

## The New Product Promise

System Pulse quietly buys users enough time to finish what they are doing before their computer gets in the way.

That promise should guide every future feature.

## Final Principle

PulseCore performs the reasoning.

System Pulse communicates the decision.

The user should never experience the complexity that PulseCore manages behind the scenes.

That is the purpose of intelligence.

That is Calm Computing.
