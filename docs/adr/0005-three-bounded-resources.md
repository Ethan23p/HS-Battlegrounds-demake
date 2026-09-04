---
status: accepted
---

# Three loosely-coupled resources, bounded rather than conserved

Value in this game moves rather than appearing from nothing, but it does so across **three
separate axes — Economy, Power, and Minions** — which are technically distinct, loosely
correlated, and convertible only slowly or at a cost. The design inspiration is a game like
Hades, where currencies are nominally liquid but exchange badly enough that each retains
its own character and its own decisions.

Conservation is **bounded, not strict**: large gains are taxed and losses are limited.
Value is therefore damped rather than preserved — the system cannot run away upward, and
cannot collapse downward, without either bound being a hard wall.

## Consequences

- **Three axes give three kinds of decision.** A single currency would collapse every
  choice into one exchange rate; three loosely-coupled ones mean "I am rich but weak" and
  "I am strong but out of bodies" are genuinely different problems.
- **Damping replaces balance patching.** A sublinear curve on gains bounds scaling
  structurally, so no Ability needs a hand-tuned cap to stay sane. Every Effect's magnitude
  passes through the same function.
- **"Conservation" is the wrong word for what this literally does**, and we are keeping it
  anyway as the name of the *intent*. Taxed value does not move somewhere else; it is
  removed. Anyone reading the code expecting a balance equation will not find one.
- Scope is **in-run**: the three axes reset each Run. A meta scope outside the Run is
  anticipated but explicitly out of scope for the prototype.

## Open

What Economy, Power and the third axis precisely *are*; what the conversion paths between
them are and what each costs; and the shape of the tax and loss-limit curves. The
principle is settled; every number and mechanism is not.
