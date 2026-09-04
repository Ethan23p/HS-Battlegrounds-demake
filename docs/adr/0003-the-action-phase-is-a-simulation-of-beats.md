---
status: accepted
---

# The Action Phase is a simulation of Beats, not a sequence of actions

In Battlegrounds, combat alternates: one side attacks, then the other, with a coin flip
breaking the tie over who starts. Whether a Minion ever acts depends on whether something
killed it first, so outcomes hinge on an ordering players cannot fully see.

We resolve the Action Phase in **Beats** instead. Everything resolving within a Beat
resolves simultaneously, against the state at the Beat's opening; deaths apply at its end;
Beats are strictly ordered. The framing matters as much as the mechanism: an auto-battler's
combat is *a simulation playing out*, and decomposing it into discrete per-minion actions
is a holdover from card games and tabletop RPGs, not something the form requires. A Beat
is a slice of simulated time, not a turn anybody takes.

These two ideas are compatible and it is worth saying why, since they can read as
contradictory: the narrative is sequential (Beat follows Beat, one moment at a time) while
resolution within any single moment is simultaneous. You watch it unfold; nobody goes first.

## Consequences

- **The opening coin flip disappears**, along with a large share of the Action Phase's
  variance. Much of Battlegrounds' randomness is really just "who swung first".
- **Trades become mutual.** Two 3/3s kill each other; neither side gains by acting first,
  because there is no first. This changes the value of nearly every stat line, and
  intuitions carried from Battlegrounds about what makes a good Board are suspect.
- **A Beat is a pure function** from world-state to world-state, which is dramatically
  easier to test, explain, and render than an interleaved attack sequence — and directly
  serves the goal of unambiguous resolution.
- **The renderer inherits a natural pacing.** Beats are what a frontend animates, one at a
  time, without the engine knowing anything about display.
- Keyword semantics need re-derivation rather than translation. Divine Shield, Poisonous
  and Windfury were balanced against sequential resolution; at minimum Poisonous is
  markedly stronger when every exchange is mutual, and Windfury's "attacks twice" has no
  turn to take twice and must mean something else.

## Open

Whether every Minion acts in every Beat, or whether Minions have differing rates so that a
Beat advances a clock and only what has come due resolves. The second reading follows more
naturally from "simulation rather than actions" and would give Windfury an obvious meaning
(a faster rate), at the cost of introducing time as a dimension. Unresolved, and it is the
gate on v0.1.
