---
status: accepted
---

# A Run against a stream of opposing Parties

One **Player**, playing a **Run**, faces a stream of opposing Parties drawn from a pool —
no second Seat. Supersedes [ADR 0002](0002-two-seats-not-eight.md): once Prep became
unbounded and the opponent a Party drawn in advance ([ADR 0004](0004-asynchronous-matches.md)),
the second Seat had nothing left to do.

## Consequences

- **The engine models one Player**; an opposing Party is an input, not an actor.
- **Reads to Claude as a roguelike shape** — resources scoped to a Run, unbounded Prep,
  offline play, opponents as stored data. A genre label Claude is applying, not one
  Ethan used; his own reference point was *"Hades"* ([0002](../transcripts/0002-vocabulary-and-action-phase.md)).
- **Difficulty must be authored, not emergent** — Battlegrounds gets its curve free from
  seven players improving in parallel; a stream needs its curve designed in.
- Symmetry is lost as a correctness check — against a stream, an unfair rule just reads
  as difficulty.

## Open

Where opposing Parties come from, and how their difficulty scales across a Run.
