# System Pulse Decision Architecture

Date: 1 July 2026

## Purpose

This note records the product direction discovered during founder UAT after the first menu bar and Today builds.

It does not replace the Product Bible. It clarifies how the app should behave as it becomes more useful.

## Principle

System Pulse is not a window into your computer.

It is a control panel for protecting your momentum.

## Interaction Model

The popup reassures.

The dashboard solves.

The expanded Today view should not become another system monitor. It should help the user decide whether to keep working, wait until a natural break, or approve a specific care action.

## Card Rule

Every card should do one of three things:

- reassure the user,
- offer one useful action,
- disappear.

If a card does not help the user make a decision, it should not be visible by default.

## Applications Rule

Every application must answer:

Can I help?

If the answer is no, the application should not become a noisy dashboard card.

Healthy applications belong in diagnostic details, not in the main decision path.

## Care Actions

Care actions must be:

- local,
- user-approved,
- low disruption,
- reversible where practical,
- respectful of active work.

System Pulse must not perform hidden optimisation, destructive cleanup, automatic restarts, or cloud-side action.

## Active Work

PulseCore should protect active work.

When an app appears to be supporting the user's current work, the correct recommendation may be no recommendation.

## Current UAT Direction

Version One should begin with safe, explicit actions such as:

- restart a browser at a natural break,
- quit Safari when appropriate,
- restart Finder when appropriate,
- open Storage Settings,
- open Activity Monitor when investigation is safer than advice.

Future versions can consider deeper care actions only after trust is earned.
