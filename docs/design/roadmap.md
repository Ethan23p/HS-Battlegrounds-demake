# Roadmap

Three milestones. Each is defined by a thing you can *do*, not by a set of files that
exist. v0.3 is the prototype.

## v0.1 — A Combat resolves

Hand the engine two Boards; it fights them and tells you what happened.

- Minion Definitions load from data files.
- Combat runs to completion: Ticks, simultaneous resolution, deaths, Deathrattles,
  cascading Triggers, keyword interactions.
- The same Seed and the same two Boards always produce the same result, and a readable
  log explains every step of it.
- No gold, no Shop, no Seats taking Actions, no bots. Boards are constructed directly.

Why first: Combat is where the hard problems live — trigger cascades, and three of the
five capabilities the [card survey](../research/card-shape-survey.md) identified as
resisting simple data (adjacency, overkill, kill attribution). It is also the only part
testable in complete isolation.

## v0.2 — A Round completes

A Seat plays a Recruit Phase, then that Board goes to Combat.

- Gold, Shop, Pool, Tavern Tier.
- Actions: buy, sell, reroll, freeze, reposition, tier up, end.
- Conservation of value enforced and observable.
- Still headless: a bot occupies the Seat.

## v0.3 — A Match completes — **the prototype**

Two Seats play Rounds until one wins.

- Both Seats occupied, by bots or by a human through the terminal frontend.
- Health, damage on loss, victory.
- Enough of a frontend to play a full Match by hand and enough logging to read one back.

At this point the determinism requirement pays out: thousands of seeded Matches can be run
unattended, and any rule bug becomes a reproducible Seed rather than an anecdote.
