# Roadmap

Three milestones. Each is defined by a thing you can *do*, not by a set of files that
exist. v0.3 is the prototype.

## v0.1 — An Action Phase resolves

Hand the engine two Parties; it fights them and tells you what happened.

- Unit Definitions load from data files.
- The Action Phase runs to completion: Beats, the left-to-right sweep, deaths, Deathrattles,
  cascading Triggers, keyword interactions.
- The same Seed and the same two Parties always produce the same result, and a readable
  log explains every step of it.
- No gold, no Shop, no Players taking Actions, no bots. Parties are constructed directly.

Why first: The Action Phase is where the hard problems live — trigger cascades, and three of the
five capabilities the [card survey](../research/card-shape-survey.md) identified as
resisting simple data (adjacency, overkill, kill attribution). It is also the only part
testable in complete isolation.

## v0.2 — A Round completes

A Player plays a Prep Phase, then that Party goes to the Action Phase.

- Gold, Shop, Pool, Tier.
- Actions: buy, sell, reroll, freeze, reposition, tier up, end.
- Conservation of value enforced and observable.
- Still headless: a bot occupies the Player.

## v0.3 — A Run completes — **the prototype**

Two Players play Rounds until one wins.

- Both Players occupied, by bots or by a human through the terminal frontend.
- Health, damage on loss, victory.
- Enough of a frontend to play a full Run by hand and enough logging to read one back.

At this point the determinism requirement pays out: thousands of seeded Runes can be run
unattended, and any rule bug becomes a reproducible Seed rather than an anecdote.
