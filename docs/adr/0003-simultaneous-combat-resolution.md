---
status: accepted
---

# Combat resolves simultaneously

In Battlegrounds, Combat alternates: one side attacks, then the other, with a coin flip
breaking the tie over who starts when boards are the same size. Whether a Minion ever gets
to act depends on whether something killed it first, so outcomes hinge on ordering that
players cannot fully see. We instead resolve each **Tick** simultaneously: every attack is
computed against the state at the start of the Tick, and all resulting deaths are applied
at its end. Rock-paper-scissors, not a turn order.

## Consequences

- **The opening coin flip disappears**, along with a large share of Combat's variance. A
  significant amount of Battlegrounds' randomness is really just "who swung first".
- **Trades become mutual.** Two 3/3s kill each other; neither side gains by acting first,
  because there is no first. This changes the value of nearly every stat line, and any
  intuitions carried over from Battlegrounds about what is a good board are suspect.
- **A Combat becomes a sequence of discrete, independently checkable states.** Each Tick is
  a pure function from board-pair to board-pair, which is dramatically easier to test,
  explain, and render than an interleaved attack sequence — and directly serves the goal
  of unambiguous resolution.
- Keyword semantics need re-derivation rather than translation. Divine Shield, Poisonous
  and Windfury were all balanced against sequential resolution, and at least Poisonous
  becomes markedly stronger when every exchange is mutual.

## Open

The *decision* is settled; the mechanism is not. Still to design: how attackers are paired
with defenders within a Tick, whether every Minion attacks every Tick, how Taunt
constrains pairing when both sides choose at once, and how Windfury expresses "twice" when
there is no turn to take twice.
