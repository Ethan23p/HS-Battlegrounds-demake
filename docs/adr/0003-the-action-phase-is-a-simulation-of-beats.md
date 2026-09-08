---
status: accepted, partially superseded by ADR-0008
---

# The Action Phase is a left-to-right sweep of Beats

> **Partially superseded — and the superseded parts are Claude's invention, not
> Ethan's.** He asked for exactly one change: *"instead of resolving combat back and
> forth in turns, I'd like each turn of attacks to resolve simultaneously"*
> ([0001](../transcripts/0001-project-kickoff.md)). Everything struck through below —
> fixed slot-vs-slot targeting, Taunt and Windfury needing redefinition, combat losing
> its randomness entirely — is a previous Claude instance reading "simultaneous" as
> license to invent all of that on top, then writing it down as though it followed from
> the request. It didn't, and he didn't ask for it. His own summary of the correction:
> *"synchronous beats simply means there's no meaningful distinction in the order that
> attacks are issued - attack targeting remains as it is in Battlegrounds, random except
> for taunt and special circumstances"* (this session, 2026-09-08). See
> [ADR 0008](0008-targeting-is-random-simultaneity-is-the-only-delta.md) for the full
> trace of where the drift happened, the correction, and what's actually in the engine.
> What's genuinely kept from this ADR — none of it touching targeting: a Beat as the unit
> of advancement, deaths applying at the end of the Beat that caused them, and the sweep
> repeating until a Party empties.

Battlegrounds alternates attacks, with a coin flip breaking who starts; whether a Unit
acts depends on an ordering players can't fully see. Instead, the Action Phase **sweeps
Slot by Slot, left to right**, and the two facing Units resolve **synchronously** — that
moment is a **Beat**. Combat is a simulation playing out, not discrete per-unit actions;
the sweep keeps the narrative sequential while resolution within a moment stays
symmetric.

## Consequences

- **The opening coin flip disappears**, and with it most of the Action Phase's variance
  — much of Battlegrounds' randomness is really "who swung first." *(This one holds.)*
- ~~No targeting decision exists. Slot *i* faces Slot *i*, removing all target-selection
  randomness and making ordering the Party the central skill.~~ **Claude's invention, not
  Ethan's request** — see the callout above. Targeting is random, exactly as in
  Battlegrounds; see ADR 0008.
- **Trades become mutual** — two 3/3s facing each other kill each other. *(Still true when
  two attackers happen to target each other; no longer guaranteed every Beat.)*
- **A Beat is a pure function** of world-state, trivially testable, and it hands the
  frontend its pacing.
- Poisonous is stronger when every exchange is mutual.
- ~~The keywords survive, redefined: Windfury is a Beat's action done twice; Taunt
  becomes protection of neighbours rather than a redirect. Its exact rule is
  unsettled.~~ **Claude's invention, not Ethan's request.** Neither keyword needed
  redefining; both already meant exactly what Battlegrounds means.

## The sweep, precisely

- The sweep **repeats** from Slot 1 after Slot 8 — a single pass would leave health
  nearly meaningless.
- Deaths apply at the **end of the Beat** that caused them, so a Slot-3 kill is gone by
  Slot 4.
- ~~A Slot with only one side has that Unit strike the Player directly — Battlegrounds'
  damage-on-loss, relocated.~~ **Claude's invention, not Ethan's request, and not even a
  Battlegrounds mechanic** — a byproduct of the slot-pairing invented above. Removed; see
  ADR 0008.

## Open

Taunt's exact positional rule. *(Resolved by ADR 0008 — Taunt isn't positional; it
constrains random targeting, exactly as in Battlegrounds.)*
