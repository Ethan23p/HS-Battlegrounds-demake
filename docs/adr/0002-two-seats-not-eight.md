---
status: superseded by ADR-0006
---

# Two Seats, not eight

Battlegrounds is an eight-player free-for-all: Seats pair off each Round, get
eliminated, and compete over one shared Pool. We're building a one-on-one Match instead —
the lobby machinery (pairing, elimination, ghosts, placement) sits around the game, not
inside it.

## Consequences

- The shared Pool becomes a two-party question. Pool scarcity is a real strategic layer
  of Battlegrounds, and we lose most of it for now.
- No matchmaking, elimination, ghosts, or placement.
- Cheap to revisit: Seats are indexed, not named `player`/`opponent`, so raising the
  count is a setup change, not a rewrite.

## Superseded

Asynchrony ate the seat model: once Prep became unbounded and the opponent a Party drawn
in advance ([ADR 0004](0004-asynchronous-matches.md)), a second Seat had nothing left to
do. What survives is the reasoning about the lobby; what doesn't is "two Seats" — see
[ADR 0006](0006-a-run-against-a-stream.md).
