# Roadmap

Three milestones, each defined by a thing you can *do*. v0.3 is the prototype. The
boundaries are Ethan's own answer to "what's the first moment you'd call it working?":
*"Let's say v0.1 is A, v0.2 is B, v0.3 is C and the official 'prototype'"*
([0002](../transcripts/0002-vocabulary-and-action-phase.md)).

## v0.1 — An Action Phase resolves

Hand the engine two Parties; it fights them and reports what happened.

- Unit Definitions load from data files.
- The Action Phase runs to completion: Beats, Passes, deaths, Deathrattles, cascading
  Triggers, keyword interactions.
- Same Seed + Parties always produce the same result, with a readable log.
- No gold, Shop, Player actions, or bots — Parties are constructed directly.

Why first: the hard problems live here — trigger cascades, and three of the five
capabilities the [card survey](../research/card-shape-survey.md) flagged as resisting
simple data — and it's the only part testable in isolation.

## v0.2 — A Round completes

A Player plays a Prep Phase, then that Party goes to the Action Phase.

- Gold, Shop, Pool, Tier.
- Actions: buy, sell, reroll, freeze, reposition, tier up, end.
- Conservation of value enforced and observable.
- Still headless: a bot occupies the Player.

## v0.3 — A Run completes — the prototype

Two Players play Rounds until one wins.

- Both occupied, by bots or a human via a frontend — TypeScript is the stated technology
  (2026-09-08); not yet built, and `bg-cli`'s role relative to it is still open (see
  [the scratchpad](../scratchpad/state.md)).
- Health, damage on loss, victory.
- Enough frontend to play a full Run by hand, enough logging to read one back.

Determinism pays out: thousands of seeded Runs run unattended, and a rule bug becomes an
anecdote no longer — just a reproducible Seed.
