---
status: accepted
---

# The Party is left-anchored, and a Beat is a time-step

Neither Hearthstone nor Battlegrounds has a clean answer to where Units sit. Hearthstone
keeps a board *centred*, so playing or losing a minion shifts everything; Battlegrounds is
left-anchored by convention but its engine still tethers positions in places. Both leave a
player unable to say, from the board alone, who acts next. Since ordering the Party is
supposed to be a real decision here, that ambiguity is worth designing out.

**The rule itself is Ethan's, not Claude's:** *"Instead of Battleground's ambiguity about
positioning, this app will anchor the party on the left-most unit and occasionally
compact toward them. Compaction is persistently applied in the prep phase, then applied
scarcely in the action phase"* ([0003](../transcripts/0003-action-phase-corrections.md)).
Everything below this point — the Beat/Pass mechanism and the **Consequences** section —
is Claude's engineering of that rule, not further instructions from him.

Two rules do it.

**A Party is anchored on its left-most Unit, and closes ranks toward it.** Persistently
during the Prep Phase — there is never a gap to reason about while arranging, and the
Party that leaves the Prep Phase is packed by construction. Scarcely during the Action
Phase — exactly once per Pass, at Beat 0.

**A Beat is a time-step of the Board**, not a turn anybody takes and not a step through
any one Party. The clock runs in **Passes** of nine Beats:

| Beat | What resolves |
|---|---|
| 0 | Both Parties close ranks |
| 1–8 | Slot 1 through Slot 8 — the Unit standing there acts, on both sides at once |

So **Beat *n* resolves Slot *n***: the clock reading and the position are the same number,
and Slots are numbered 1 through 8 to make that true. A Slot nobody occupies has nobody to
act. Within a Pass, a Unit that dies leaves its Slot empty and the Slot *stays* empty; the
clock passes over it.

## Consequences

- **Nothing moves under the clock mid-Pass.** The order for a Pass is fixed the moment it
  begins, so it can be read straight off the Board. A death removes that Unit's Beat and
  nothing else — it never grants a Beat to a Unit that already acted, nor steals one from
  a Unit that hasn't. That is exactly the class of quirk this replaces.
- **The clock belongs to the Board, not to either Party.** Both sides are somewhere in the
  same Beat always, which is what keeps simultaneity ([ADR 0008](0008-targeting-is-random-simultaneity-is-the-only-delta.md))
  meaningful when the Parties are different sizes. Nothing here is per-Party except which
  Units happen to stand where.
- **A hole is legible state.** An empty interior Slot means "that Unit died earlier this
  Pass", visible rather than bookkept, and it closes at a moment the player can predict.
- **Compaction is a Board change, so it is an Event.** `Compacted` is logged whenever it
  actually moves something — true today, verifiable in `crates/bg-sim`.
- Two more consequences Claude expected from this rule — about summons and about
  `Selector::Adjacent` — involve systems that don't exist yet (Effects aren't executed;
  see the roadmap). Moved to
  [the scratchpad](../scratchpad/state.md#design-notes-for-unbuilt-systems) as
  expectations rather than verified consequences, so this ADR doesn't assert things about
  code that isn't written.
- Closing ranks every Pass is also what guarantees termination: a non-empty Party always
  re-packs to Slot 1, so a Pass with anyone alive in it always has a Beat that does
  something.

## Vocabulary

**Pass** is provisional — Ethan's phrase was "a round of beats", but **Round** is already
taken (one Prep Phase plus one Action Phase), so the word could not be reused. *Sweep*,
from the superseded [ADR 0003](0003-the-action-phase-is-a-simulation-of-beats.md), was
deliberately not revived: it named a mechanism this one replaces. Rename freely.
