---
status: accepted
---

# The Action Phase is a left-to-right sweep of Beats

In Battlegrounds, combat alternates: one side attacks, then the other, with a coin flip
breaking the tie over who starts. Whether a Unit ever acts depends on whether something
killed it first, so outcomes hinge on an ordering players cannot fully see.

Instead, the Action Phase **sweeps Slot by Slot from left to right**. At each Slot, the two
Units facing each other resolve **synchronously**. That moment is a **Beat**.

Two ideas are doing work here and they are easy to confuse. The first is a framing: an
auto-battler's combat is *a simulation playing out*, and carving it into discrete
per-unit actions is a holdover from card games and tabletop, not something the form
demands. The second is the mechanism that serves it: a positional sweep, so the narrative
is sequential (Beat follows Beat, left to right, one moment at a time) while resolution
within any single moment is symmetric. You watch it unfold; in any given Slot, nobody
goes first.

## Consequences

- **The opening coin flip disappears**, along with a large share of the Action Phase's
  variance. Much of Battlegrounds' randomness is really just "who swung first".
- **There is no targeting decision at all.** Slot *i* faces Slot *i*. Every scrap of
  target-selection randomness leaves the game, which is the strongest possible service to
  unambiguous resolution — and it makes **ordering the Party the central skill** of the
  Prep Phase, giving the shopping half real depth with no extra machinery.
- **Trades become mutual.** Two 3/3s facing each other kill each other. Intuitions carried
  from Battlegrounds about what makes a good board are suspect.
- **A Beat is a pure function** from world-state to world-state, trivially testable, and
  it hands the frontend its pacing: a renderer animates Beats, in order, knowing nothing
  else.
- **Taunt has nothing left to do.** The keyword exists to constrain a choice of target, and
  there is no choice. It needs reinventing or dropping. Windfury's "attacks twice" has no
  turn to take twice and likewise needs a new meaning.
- Poisonous is markedly stronger when every exchange is mutual.

## Open

Whether the sweep repeats after Slot 8 or the Action Phase is a single pass; when deaths
are applied — at the end of the Beat that caused them, or the end of the sweep; and what
happens in a Slot where only one side has a Unit.
