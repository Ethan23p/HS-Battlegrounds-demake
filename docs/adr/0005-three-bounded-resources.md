---
status: accepted
---

# Three loosely-coupled resources, bounded rather than conserved

Battlegrounds already has an economy. It exchanges currency for power, power for units, and
units for meta-progress, continuously, as an emergent property — it simply was not designed
as one, and its balance is hand-tuned around it after the fact. **Our delta is
intentionality, not the existence of the economy.**

We name three axes — **Economy, Power, Units** — and treat them as technically separate,
loosely correlated, and convertible only slowly or at a cost. Nominally liquid, practically
sticky, so each keeps its own character and its own decisions. Magnitudes follow
Battlegrounds' until there is a reason to differ, anchored on **1** as the atom: the most
basic, unexceptional unit of measure.

Conservation is **bounded, not strict**. Concretely, at the point where it currently bites:
damage in a single action is compressed — **the big actor is taxed and the small actor's
losses are cut** — so magnitudes cannot run away upward or collapse to irrelevance.

## Consequences

- **Three axes give three kinds of decision.** One currency collapses every choice into one
  exchange rate; three loosely-coupled ones make "rich but weak" and "strong but short of
  bodies" genuinely different problems.
- **Compression replaces balance patching.** A sublinear curve bounds scaling structurally,
  so no Ability needs a hand-tuned cap. It also keeps small Units relevant late, which is
  where Battlegrounds' curve tends to abandon them.
- **"Conservation" is the wrong word for what this literally does**, and we keep it as the
  name of the *intent*. Taxed value is removed, not moved. Anyone reading the code
  expecting a balance equation will not find one.
- Scope is the **Run**. A meta scope is anticipated but explicitly out of scope for the
  prototype.

## Open

Deliberately unresolved for now: the exact conversion paths between axes and what each
costs, and the shape of the compression curve. The intent is settled and the numbers start
at Battlegrounds' defaults; the dynamics get designed once the game is running and there is
something to feel.
