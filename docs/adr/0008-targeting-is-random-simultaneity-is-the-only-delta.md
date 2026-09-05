---
status: accepted
---

# Targeting is random; simultaneity is the only delta from Battlegrounds' attack order

ADR 0003 overreached. The only rule Ethan actually asked to change was *how attacks are
sequenced* — replacing Battlegrounds' alternating turns with simultaneous ones. Recorded
as "positional pairing" with "no targeting decision exists," that single delta grew, across
several rounds of the design conversation, into three unrequested ones: fixed slot-vs-slot
targeting instead of Battlegrounds' random targeting, redefinitions of Taunt and Windfury,
and an Action Phase with no randomness at all. This ADR corrects the record. Side by side:

| | Battlegrounds | This project |
|---|---|---|
| Attack order | Left to right, alternating sides | Left to right, **resolved simultaneously** |
| Targeting | Random, unless a Taunt Unit is in play | Random, unless a Taunt Unit is in play — **unchanged** |
| Taunt | While a Taunt Unit is in the opposing Party, all attacks must target it | **Unchanged** |
| Windfury | Acts twice on its turn, choosing a target each time | Acts twice in its Beat, choosing a target each time — **unchanged** |
| Attacking a Player | Never happens — that is Hearthstone, not Battlegrounds | **Unchanged** |
| Damage / death | Sequential — the surviving attacker is unambiguous | Simultaneous; **the attacker dies last** |

Only row one is a delta. Everything else is the default rule doing its job: undeltered,
it works however Battlegrounds works.

## What this reverses

- **Slot-vs-slot pairing is gone.** A Unit's target is drawn at random from the *opposing
  Party*, not from the mirrored Slot. "Slot *i* faces Slot *i*" ([CONTEXT.md](../../CONTEXT.md)'s
  old Board definition) was never something Ethan asked for.
- **Taunt and Windfury are not redefined.** Taunt constrains the random draw: while any
  Taunt Unit is alive in a Party, every attack against that Party must target one. Windfury
  is unaffected by this ADR — "the action a Unit would do once in a Beat, it does twice"
  already matched Battlegrounds and stays.
- **The Action Phase has randomness again.** Target selection draws from
  [`Domain::Combat`](../../crates/bg-sim/src/rng.rs), starting in v0.1 rather than waiting
  for Effects. `resolve` now takes an `&mut Rng`.
- **"Unopposed Unit strikes the Player" is removed**, not reinterpreted. It only existed
  because Slot-pairing could leave one side's higher Slots facing nothing while both
  Parties were still alive overall. Under random targeting that situation doesn't arise:
  a side's Party is either alive (and a full target pool) or empty (and the Action Phase
  has already ended). Damage-on-loss, as a single end-of-fight calculation, stays exactly
  where the roadmap already puts it — v0.2/v0.3, computed from the survivors on
  `Resolution::final_board`.

## The mechanism

A **Beat** is a time-step of the Board, and Beat *n* resolves Slot *n* — on both sides at
once. That is simultaneity's entire meaning, and the only thing this ADR asks the engine
to do differently from Battlegrounds. [ADR 0009](0009-the-party-is-left-anchored.md)
specifies the clock those Beats run on, and when a Party closes ranks around its dead.

A Beat proceeds attack by attack (Windfury's second attack is a second instance). Within
an instance, both sides' current attacker strike at once: **each draws its own target**
from the opposing Party's living Units, respecting Taunt. Damage and Poisonous apply, and
removal is deferred to the end of the Beat — so a Unit fatally wounded in instance 0 is
still a valid attacker, but not a valid *target*, for instance 1. An attack that finds
nothing left standing does not land: no Unit ever attacks a Player.

**Attacker dies last:** when a Beat's deaths are applied, any Unit that died *without*
attacking this Beat is removed first; a Unit that attacked and also died this Beat is
removed after. This keeps a kill attributable to its attacker even when the trade was
mutual, which matters once Effects can ask "did I kill something this Beat" (the
[card survey](../research/card-shape-survey.md)'s kill-attribution capability).

## Two calls that were flagged, and how they landed

This ADR originally invented two mechanics that weren't in the table above, flagged them
as engineering calls, and got both corrected — worth keeping on the record, since the
whole reason this ADR exists is unflagged invention drifting into settled rules.

- **An attack with no target left striking the Player: rejected.** It read as
  Battlegrounds' behaviour and isn't — Hearthstone lets a minion go face, Battlegrounds
  never does. Such an attack now simply doesn't land, and `Resolution` carries no
  damage-to-Player totals at all.
- **`turn_count % party.len()` cycling: replaced**, by the Pass and its compaction rule in
  [ADR 0009](0009-the-party-is-left-anchored.md). The modulo scheme quietly assumed the
  Party re-packs after every Beat, which is exactly the positional ambiguity 0009 sets out
  to remove.

## Superseded

[ADR 0003](0003-the-action-phase-is-a-simulation-of-beats.md)'s framing (a Beat as the
unit the Action Phase advances by, deaths at the end of the Beat that caused them, the
sweep repeating until a Party is empty) still holds. Its claims about targeting,
determinism, and the keyword casualties do not; this ADR replaces those specifically.
