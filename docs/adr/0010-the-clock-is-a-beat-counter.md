---
status: accepted
---

# The clock is a Beat counter, and readiness rides on the Unit

[ADR 0009](0009-the-party-is-left-anchored.md) removed Battlegrounds' positional
ambiguity by holding the Party still: Slots kept their gaps for the length of a Pass, and
Beat *n* resolved Slot *n*. It works, but it pays for stillness with state a reader has to
carry — a Pass above the Beat, a Beat 0 that is not a Slot, and an empty Slot that means
"someone died earlier in this Pass" rather than "nobody is here". The clock had to be
told about the Party's history to know who acts next.

Ethan's correction is to invert that. Time is one counter, position is relative, and a
Beat is judged by the difference it made:

> *the passage of time only happens through beats, beats progress at a consistent rate in
> one direction within a round (they don't reset), there's no concept of "now we are
> looping back to the start" but there should be a concept of "we've ennumerated through
> all party members"* ... *instead of focusing on absolute values (SLOT-3 = EMPTY) let's
> place things relative to each other (the third unit died); instead of focusing on which
> action happened first, let's measure the state after the beat against the state from
> before the beat; instead of triggering on 'start/end of pass' lets trigger on* before
> *slot 1* — [transcript 0004](../transcripts/0004-the-beat-counter.md).

## The rules

**Beats count from 1 and never reset.** One Action Phase, one clock, one direction. There
is no unit of time above the Beat, so there is nothing to reset *to*.

**A Party has no gaps.** It is a run of Units anchored on its left-most, and a Slot is
where a Unit stands in that run. `SLOT-3 = EMPTY` is not a state the type can hold: when a
Unit dies, the Party closes up in the same step that removes it. Closing ranks stops being
an operation, an Event, and a rule — a Party is never uncompacted, so nothing needs
compacting.

**Readiness rides on the Unit.** Every Unit carries whether it still owes the clock a
turn. Each Beat, the left-most **Ready** Unit of each Party acts — on both sides at once —
and stops being Ready. When neither Party has a Ready Unit left, every Unit becomes Ready
again.

That refresh is the only structure the clock has, and it is *before the first Unit acts*,
exactly where Ethan put the trigger. It is a condition met, not a step counted: nothing
loops back to a start, and no Beat is skipped or spent on bookkeeping.

## Why this is stronger than holding the Party still

The reason 0009 froze positions was that a cursor counting Slots is wrong the instant the
Party moves under it. Take a Party of three where the first Unit acts in Beat 1 and dies
in it:

| | Party after Beat 1 | Beat 2 resolves |
|---|---|---|
| Cursor over Slots, Party closes up | `[second, third]` | Slot 2 → **third**. "second" never acts. |
| Cursor over Slots, Party frozen (0009) | `[_, second, third]` | Slot 2 → second ✓, at the cost of gaps |
| Readiness on the Unit (this ADR) | `[second, third]` | left-most Ready → second ✓ |

Only the third column gets the right answer *and* a Party with nothing in it but Units.
The guarantee 0009 spent gaps to buy — a death never grants a Beat to a Unit that already
acted, nor steals one from a Unit that hasn't — now falls out of where the state lives.
**Nothing acts twice before everything standing has had its turn** — and a Unit that dies
before its turn comes simply never gets one, which is the only way to lose a turn.

## Consequences

- **A Beat is the unit of comparison.** Nothing is removed part-way through one, so the
  Board before a Beat and the Board after it differ by everything that Beat did. The order
  of strikes inside a Beat is bookkeeping; only the difference is rules. This is what
  Triggers will read.
- **The clock still belongs to the Board.** Both sides are always in the same Beat, which
  is what keeps simultaneity ([ADR 0008](0008-targeting-is-random-simultaneity-is-the-only-delta.md))
  meaningful across Parties of different sizes. The refresh waits for the *longer* Party,
  so a lone survivor facing five sits out four Beats between its turns.
- **Adjacency is plain Slot arithmetic**, with no "as long as nothing has moved" caveat —
  `Selector::Adjacent` and the [card survey](../research/card-shape-survey.md)'s adjacency
  capability read the Party as it stands.
- **The log lost an Event and gained one.** `Compacted` is gone; `AllReady` marks the Beat
  a refresh preceded. `Died` names the Slot a Unit stood in as it died, `Reborn` the Slot
  the returning Unit stands in now — each the position at the moment of the Event, which
  is the only position that is ever true.
- **Termination is unchanged in kind** and now counted directly: [`MAX_BEATS`] rather than
  a cap on Passes.
- **1-based Slots are no longer load-bearing.** They were numbered 1–8 so that "Beat *n*
  resolves Slot *n*" could be true. It is not a rule any more; the numbering stays because
  it is how the rules, the log and Ethan all count, not because arithmetic depends on it.

## Open: does a Unit arrive Ready?

Once Effects can summon, a Token entering play mid-enumeration is either Ready (and acts
before the refresh) or not (and waits for it). 0009 answered this positionally — ahead of
the clock acts, behind it waits — and that answer is gone with the gaps.

Nothing can summon yet, so nothing observes this today; the decision belongs with
[ADR 0007](0007-abilities-are-data-with-a-modifier-stage.md)'s Effects. The recommendation
is **not Ready**: it keeps "nothing acts twice before everything standing has had its
turn" literally true, and it stops a summon chain from extending an enumeration
indefinitely. Battlegrounds is closer to the other answer, so
this is a delta if taken, and Ethan's call.

## Supersedes

[ADR 0009](0009-the-party-is-left-anchored.md)'s mechanism. Its purpose stands and is
better served: the Party is still anchored on its left-most Unit, and you can still say
from the Board alone who acts next. What goes is Pass, Beat 0, closing ranks as an
operation, and the empty interior Slot. **Pass** as vocabulary is retired rather than
renamed — there was never anything for it to name but "nine Beats", and there are no
nine-Beat groups any more.
