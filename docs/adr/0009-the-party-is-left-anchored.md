---
status: accepted
---

# The Party is left-anchored, and compaction is scarce during the Action Phase

Neither Hearthstone nor Battlegrounds has a clean answer to where Units sit. Hearthstone
keeps a board *centred*, so playing or losing a minion shifts everything; Battlegrounds is
left-anchored by convention but its engine still tethers positions in places. Both leave a
player unable to say, from the board alone, who acts next. Since ordering the Party is
supposed to be a real decision here, that ambiguity is worth designing out.

**A Party is anchored on its left-most Unit, and closes ranks toward it.** When it closes
ranks is the whole of the rule:

- **Prep Phase: persistently.** The Party is left-packed at all times. There is never a
  gap to reason about while arranging, and the Party that leaves the Prep Phase is packed
  by construction.
- **Action Phase: scarcely.** Only at the start of a **Pass** — one full left-to-right
  traverse of a Party, giving each of its Units a turn — which includes before the first
  Unit attacks. Within a Pass, a Unit that dies leaves its Slot empty and the Slot *stays*
  empty; the traverse skips it.

Each side runs its own Pass over its own Party, so the two sides re-anchor independently.

## Consequences

- **Nothing moves under the cursor mid-Pass.** The attack order for a Pass is fixed the
  moment it begins, so it can be read straight off the Board. A death removes that Unit's
  turn and nothing else — it never grants a turn to a Unit that already acted, nor steals
  one from a Unit that hasn't. That is exactly the class of quirk this replaces.
- **A hole is legible state.** An empty interior Slot means "that Unit died earlier this
  Pass," visible rather than bookkept, and it closes at a moment the player can predict.
- **Summons get a free answer.** Once Effects land, a Token arriving left of the cursor
  waits for the next Pass; one arriving to its right acts in this one. No special case,
  and no ambiguity of the kind Battlegrounds has here.
- **Adjacency holds still for a Pass**, which is what makes `Selector::Adjacent` (and the
  [card survey](../research/card-shape-survey.md)'s adjacency capability) cheap: it is
  Slot arithmetic against an arrangement that isn't moving.
- **Compaction is a Board change, so it is an Event.** The log is the frontend's only
  source of truth for animating a fight, and Units sliding left is something a viewer
  sees; `Compacted` is logged whenever it actually moves something.
- Re-anchoring at every Pass boundary is also what guarantees termination: a non-empty
  Party always re-packs to Slot 0, so the traverse always finds a live Unit.

## Vocabulary

**Pass** is provisional — Ethan's phrase was "a new round of beats", but **Round** is
already taken (one Prep Phase plus one Action Phase), so the word could not be reused.
*Sweep*, from the superseded [ADR 0003](0003-the-action-phase-is-a-simulation-of-beats.md),
was deliberately not revived: it named a joint traverse of both Boards, and this is
per-Party. Rename freely.
