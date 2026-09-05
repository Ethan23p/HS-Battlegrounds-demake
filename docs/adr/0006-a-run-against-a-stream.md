---
status: accepted
---

# A Run against a stream of opposing Parties

There is one **Player**, playing a **Run**, facing a stream of opposing Parties drawn
from a pool. There is no second Seat. This supersedes
[ADR 0002](0002-two-seats-not-eight.md), which had reduced Battlegrounds' eight-player
free-for-all to a two-Seat contest.

The reduction happened on its own: once the Prep Phase became unbounded and the opponent
was drawn in advance ([ADR 0004](0004-asynchronous-matches.md)), the second Seat had
nothing left to do — it never waits, never responds, and is data by the time the Action
Phase runs. Keeping it would mean maintaining a participant that never participates.

## Consequences

- **The engine models one Player**, and an opposing Party is an input, not an actor.
  "The other Seat" in code is a bug against this ADR.
- **This is a roguelike shape**, fitting the rest: resources scoped to a Run, unbounded
  Prep, offline play, opponents as stored data. Ethan's reference point on resources was
  Hades — the same shape.
- **Difficulty must be authored rather than emergent.** Battlegrounds gets its difficulty
  curve free from seven other players improving in parallel; a stream needs its curve
  designed in — a real cost landing on whoever fills the pool.
- Symmetry is lost as a correctness check. In a two-sided contest a rule favouring one
  side is visible; against a stream, it's just difficulty.

## Open

Where the opposing Parties come from — recorded from earlier Runs, hand-authored,
procedurally generated, or some mix — and how their difficulty is made to scale across a
Run.
