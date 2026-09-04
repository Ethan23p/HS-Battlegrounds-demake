---
status: superseded by ADR-0006
---

# Two Seats, not eight

Hearthstone: Battlegrounds is an eight-player free-for-all: Seats are paired off each
Round by a matchmaker, eliminated as they die, and compete over one shared Pool. We are
building a one-on-one Match instead. The lobby is a large amount of machinery — pairing,
elimination order, ghost opponents on odd counts, placement scoring — sitting *around* the
game rather than inside it, and none of it is needed to make Recruit Phases and Combats
interesting.

## Consequences

- The shared Pool becomes a two-party question rather than an eight-party one. Pool
  scarcity — noticing that a rival is buying the Murlocs you need — is a genuine strategic
  layer of Battlegrounds, and we are choosing to lose most of it for now.
- No matchmaking, elimination, ghosts, or placement.
- This is a scope decision, not an architectural one, and it is deliberately cheap to
  revisit: Seats are indexed rather than named `player`/`opponent`, so raising the count is
  a change to match setup and pairing, not a rewrite of Combat or the Recruit Phase.
  Anything that hard-codes "the other Seat" is a bug against this ADR.

## Superseded

Asynchrony ate the seat model. Once the Prep Phase became unbounded and the opponent
became a Party drawn in advance ([ADR 0004](0004-asynchronous-matches.md)), there was
nothing left for a second Seat to do: it never waits, never responds, and by fight time is
already data. What survives of this ADR is its reasoning about the lobby, which still
applies. What does not survive is "two Seats" — there is one Player and a stream. See
[ADR 0006](0006-a-run-against-a-stream.md).
