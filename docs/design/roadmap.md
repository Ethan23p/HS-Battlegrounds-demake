# Roadmap

Three milestones, each defined by a thing you can *do*, not files that exist. v0.3 is the
prototype.

## v0.1 — An Action Phase resolves

Hand the engine two Parties; it fights them and reports what happened.

- Unit Definitions load from data files.
- The Action Phase runs to completion: Beats, the left-to-right sweep, deaths,
  Deathrattles, cascading Triggers, keyword interactions.
- Same Seed + Parties always produce the same result, with a readable log of every step.
- No gold, Shop, Player actions, or bots — Parties are constructed directly.

Why first: this is where the hard problems live — trigger cascades, and three of the five
capabilities the [card survey](../research/card-shape-survey.md) flagged as resisting
simple data (adjacency, overkill, kill attribution) — and it's the only part testable in
isolation.

## v0.2 — A Round completes

A Player plays a Prep Phase, then that Party goes to the Action Phase.

- Gold, Shop, Pool, Tier.
- Actions: buy, sell, reroll, freeze, reposition, tier up, end.
- Conservation of value enforced and observable.
- Still headless: a bot occupies the Player.

## v0.3 — A Run completes — **the prototype**

Two Players play Rounds until one wins.

- Both Players occupied, by bots or a human via the terminal frontend.
- Health, damage on loss, victory.
- Enough frontend to play a full Run by hand and enough logging to read one back.

Determinism pays out here: thousands of seeded Runs run unattended, and a rule bug
becomes a reproducible Seed rather than an anecdote.
