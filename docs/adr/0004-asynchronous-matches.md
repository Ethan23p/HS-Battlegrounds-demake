---
status: accepted
---

# Matches are asynchronous

A Seat's Prep Phase is **unbounded** — no timer, it ends when the Seat ends it. Its
opponent for the Round is **drawn from an Opponent Pool in advance**, not matched at the
moment of the fight. Ending the Prep Phase is what begins the Action Phase, for that Seat,
at that moment.

The consequence is that Seats never wait for each other and need not coexist in time at
all. This is what makes offline play work: with the opponent already chosen and its Board
already known, a complete Round needs no network, no server and no live counterpart.

## Consequences

- **"The opponent" becomes a Board, not a participant.** By the time the Action Phase runs,
  the other side is data. Nothing in the Action Phase can consult a live opponent, which is
  a real constraint on ability design — no card may ever ask the other Seat a question.
- **Unbounded Prep removes time pressure as a design tool.** Battlegrounds uses its timer
  to force imperfect decisions; we are deliberately giving that up in favour of considered
  play. Difficulty has to come from the position, not the clock.
- The engine must be able to serialise a Board well enough to fight it later. That is a
  requirement on the state model, not just on storage.
- Determinism becomes more valuable, not less: a stored opponent Board plus a Seed
  reproduces a Round exactly, which is what makes the Opponent Pool a durable artifact
  rather than a cache.

## Open

Where Opponent Pool Boards come from — recordings of other Seats' Boards, pre-authored
sets, or generated ones — and whether a Match remains a contest between two Seats with a
winner, or becomes a Run in which one Seat survives a stream of drawn opponents. The
second reading follows more naturally from asynchrony but changes what a Match *is*, and
reaches back into [ADR 0002](0002-two-seats-not-eight.md).
