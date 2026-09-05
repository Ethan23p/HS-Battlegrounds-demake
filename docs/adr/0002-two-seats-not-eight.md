---
status: superseded by ADR-0006
---

# Two Seats, not eight

Hearthstone: Battlegrounds is an eight-player free-for-all: Seats pair off each Round via
a matchmaker, get eliminated as they die, and compete over one shared Pool. We're building
a one-on-one Match instead. The lobby — pairing, elimination order, ghost opponents,
placement scoring — sits *around* the game, not inside it, and none of it is needed to
make Recruit Phases and Combats interesting.

## Consequences

- The shared Pool becomes a two-party question, not an eight-party one. Pool scarcity — a
  rival buying the Murlocs you need — is a genuine strategic layer of Battlegrounds, and
  we're losing most of it for now.
- No matchmaking, elimination, ghosts, or placement.
- A scope decision, not an architectural one, and deliberately cheap to revisit: Seats
  are indexed rather than named `player`/`opponent`, so raising the count changes match
  setup and pairing, not Combat or the Recruit Phase. Anything hard-coding "the other
  Seat" is a bug against this ADR.

## Superseded

Asynchrony ate the seat model. Once the Prep Phase became unbounded and the opponent
became a Party drawn in advance ([ADR 0004](0004-asynchronous-matches.md)), a second Seat
had nothing left to do — it never waits, never responds, and is data by fight time. What
survives is the reasoning about the lobby; what doesn't is "two Seats" — there is one
Player and a stream. See [ADR 0006](0006-a-run-against-a-stream.md).
