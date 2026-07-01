# System Pulse

# Architect's Update 003

## Protect the Session

Date: 1 July 2026

## Purpose

Founder testing revealed a critical product truth:

System Pulse is not primarily about optimising resources.

It is about protecting active work.

The user does not care that memory is high, swap is increasing, or a browser process is expensive in isolation.

They care whether the thing they are doing right now is safe to continue.

## Founder Discovery

During live testing, the most stressful moments were not caused by low metrics.

They were caused by the risk of losing the session:

- Codex becoming sluggish mid-task
- Screenshots taking a long time to attach
- Voice dictation becoming unreliable
- The fear of losing a long spoken prompt
- The fear of interrupting a long-running Codex task
- Not knowing whether restarting would destroy progress

Traditional system utilities would have recommended restarting, quitting, or clearing load.

That would have been technically correct and experientially wrong.

The highest-value behaviour was not immediate optimisation.

It was preserving the active session.

## New PulseCore Priority

PulseCore should now reason in this order:

1. Is important work currently in progress?
2. Can the user's momentum be preserved without interruption?
3. What is the lowest-disruption action available?
4. If interruption is unavoidable, when is the safest moment?

Resource improvement is secondary to session preservation.

## Preserve Before Optimise

System Pulse should favour actions such as:

- Close inactive browser windows
- Quit unused applications
- Restart Finder
- Reduce browser load
- Delay restarting Codex until the current task is complete

System Pulse should avoid recommending actions that might destroy unsaved work, interrupt generation, cancel uploads, or break user flow unless there is no safer option.

## Active Session Awareness

The product should become increasingly aware of active work states, including:

- AI generation in progress
- Voice dictation in progress
- Long-running prompts
- Large uploads or attachments
- Development builds
- Rendering or export work
- Trading, writing, or decision-heavy activity

These states should raise the cost of interruption.

## Session Risk

PulseCore should estimate not only system risk, but session risk:

How likely is this recommendation to cause the user to lose work, context, or momentum?

As session risk rises, recommendations should become more conservative.

## Product Promise

System Pulse helps users finish what they are doing before the computer gets in the way.

That promise is stronger than performance optimisation.

Protecting active work is now a first-class product principle.

## Engineering Principle

When optimisation conflicts with preserving active work, preserve the work.

Always.
