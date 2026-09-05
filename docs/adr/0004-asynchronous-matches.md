---
status: accepted
---

# Matches are asynchronous

A Seat's Prep Phase is **unbounded**, ending when the Seat ends it. Its opponent is
**drawn from an Opponent Pool in advance**, not matched at fight time — Seats never wait
for each other, which is what makes offline play work.

## Consequences

- **"The opponent" becomes a Board, not a participant** — no Ability may ever consult a
  live opponent.
- **Unbounded Prep removes time pressure** as a design tool; difficulty must come from
  the position, not the clock.
- The engine must serialise a Board well enough to fight it later.
- Determinism matters more: a stored Board plus a Seed reproduces a Round exactly,
  making the Opponent Pool a durable artifact.

## Open

Where Opponent Pool Boards come from, and whether a Match stays a two-Seat contest or
becomes a Run against a stream — see [ADR 0002](0002-two-seats-not-eight.md).
