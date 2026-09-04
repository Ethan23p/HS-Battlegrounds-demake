---
status: accepted
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
