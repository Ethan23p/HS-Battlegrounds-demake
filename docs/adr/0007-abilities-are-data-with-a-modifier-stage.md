---
status: accepted
---

# Abilities are data, resolved through a pipeline with a modifier stage

Abilities are **inert data** — a Trigger, a condition, an ordered list of Effects from a
fixed vocabulary — and the engine is their only interpreter. Adding a Unit is a data
change; adding a *kind* of Unit is the only thing touching Rust.

The [card survey](../research/card-shape-survey.md) found the top 5 ability templates
cover 53% of a sampled Battlegrounds pool, top 15 cover 81%, leaving ~30% that resists.
The residue **clusters** on five capabilities, four of them engine features (adjacency,
overkill/kill-attribution, buff-duration classes, out-of-Party reads). The fifth is
different in kind: **global rule modifiers** (Brann, Khadgar, Baron Rivendare) don't act,
they change how *other* Units' abilities resolve — a vocabulary of "on Trigger T apply
Effect E" has no slot for them, and special-casing each in Rust is how a data-driven
engine turns code-driven by attrition.

So resolution is a **pipeline** with a **modifier stage**: before an Ability's Effects
apply, registered modifiers (also data) may transform the list. "Your Deathrattles
trigger twice" duplicates effects from a matching Trigger, not a hand-written exception.
Designed in now, since retrofitting a resolution loop is much harder than building the
seam in from the start.

## Consequences

- **Units stay editable without a compiler** — the project's first requirement.
- **A second vocabulary exists** — Effects, and Modifiers over Effects — each variant
  costing a match arm, each earning its place.
- **The modifier stage is a discipline risk**: it's the obvious place to shove anything
  awkward. A modifier applying to exactly one Unit is a smell.
- **Resolution order becomes observable** — how two modifiers compose must be answered
  visibly, not left to hidden ordering.
- Four of five surveyed capabilities are cheaper here: adjacency is Slot arithmetic once
  [ADR 0003](0003-the-action-phase-is-a-simulation-of-beats.md) fixes positions; opponent
  reads are simple once opponents are data ([ADR 0006](0006-a-run-against-a-stream.md)).

## Considered and rejected

- **Pure data, no modifier stage** — simplest, but rule-modifier Units become
  impossible, and they're among the genre's most interesting cards.
- **A Rust function per Unit** — unlimited expressiveness, but every new Unit needs a
  recompile.
- **Defer the hook until demanded** — rejected because the survey measures the need at
  ~30%, and this seam is far cheaper to build in than retrofit.
