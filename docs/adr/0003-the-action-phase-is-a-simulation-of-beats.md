---
status: accepted
---

# The Action Phase is a left-to-right sweep of Beats

Battlegrounds alternates attacks, with a coin flip breaking who starts; whether a Unit
acts depends on an ordering players can't fully see. Instead, the Action Phase **sweeps
Slot by Slot, left to right**, and the two facing Units resolve **synchronously** — that
moment is a **Beat**. Combat is a simulation playing out, not discrete per-unit actions;
the sweep keeps the narrative sequential while resolution within a moment stays
symmetric.

## Consequences

- **The opening coin flip disappears**, and with it most of the Action Phase's variance
  — much of Battlegrounds' randomness is really "who swung first."
- **No targeting decision exists.** Slot *i* faces Slot *i*, removing all
  target-selection randomness and making **ordering the Party the central skill**.
- **Trades become mutual** — two 3/3s facing each other kill each other.
- **A Beat is a pure function** of world-state, trivially testable, and it hands the
  frontend its pacing.
- Poisonous is stronger when every exchange is mutual.
- **The keywords survive.** Windfury is a Beat's action done twice; Taunt becomes
  protection of neighbours rather than a redirect, since position remains even without
  target choice. Its exact rule is unsettled.

## The sweep, precisely

- The sweep **repeats** from Slot 1 after Slot 8 — a single pass would leave health
  nearly meaningless.
- Deaths apply at the **end of the Beat** that caused them, so a Slot-3 kill is gone by
  Slot 4.
- A Slot with only one side has that Unit **strike the Player directly** —
  Battlegrounds' damage-on-loss, relocated. *(Provisional.)*

## Open

Taunt's exact positional rule.
