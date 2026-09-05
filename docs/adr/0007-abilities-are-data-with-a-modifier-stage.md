---
status: accepted
---

# Abilities are data, resolved through a pipeline with a modifier stage

Abilities are **inert data** — a Trigger, a condition, and an ordered list of Effects
drawn from a fixed vocabulary — and the engine is their only interpreter. Adding a Unit
is a data change; adding a *kind* of Unit is the only thing that touches Rust.

The [card survey](../research/card-shape-survey.md) tested the usual argument for this
and found it weaker than assumed: the top five ability templates cover 53% of a sampled
Battlegrounds pool and the top fifteen cover 81%, leaving ~30% that resists
parameterisation. Crucially, the residue **clusters** on five named capabilities rather
than scattering. Four of the five are engine features (adjacency, resolution internals
like overkill and kill attribution, buff-duration classes, reads of out-of-Party state).
The fifth is different in kind, and it's what this ADR is really about.

**Global rule modifiers** — Battlegrounds' Brann, Khadgar, Baron Rivendare — don't *do*
anything. They change how *other* Units' abilities resolve. A vocabulary that only says
"on Trigger T, apply Effects E to Selector S" has nowhere to put them, and the tempting
fix is to special-case each one in Rust, which is how a data-driven engine becomes a
code-driven one by attrition.

So resolution is a **pipeline**, with a **modifier stage**: before an Ability's Effects
are applied, registered modifiers may transform the list. Modifiers are themselves
declared in data, from their own small closed vocabulary. "Your Deathrattles trigger
twice" is a modifier that duplicates effects from a matching Trigger — not a hand-written
exception.

The stage is designed in now rather than discovered later, because retrofitting a
transformation point into a resolution loop is far harder than building one in, and the
survey says the demand is real rather than hypothetical.

## Consequences

- **Units stay editable without a compiler**, the requirement from the project's first
  sentence, and a Unit editor or generated content stays possible.
- **A second vocabulary now exists** — Effects, and Modifiers over Effects — and both
  cost a match arm per variant. Each must earn its place by expressing something a
  combination of existing ones cannot.
- **The modifier stage is a discipline risk.** It's the obvious place to shove anything
  awkward, and doing so would recreate per-Unit special-casing inside a stage that merely
  looks principled. A modifier that applies to exactly one Unit is a smell; the
  vocabulary should describe a *kind* of rule change.
- **Resolution order becomes observable.** With modifiers able to duplicate and
  transform effects, "what order do two modifiers compose in" is a real question the
  rules must answer, and answer visibly — nothing in this project should hinge on hidden
  ordering.
- Four of the five surveyed capabilities are cheaper here than in Battlegrounds:
  adjacency is Slot arithmetic once [ADR 0003](0003-the-action-phase-is-a-simulation-of-beats.md)
  fixes positions; cross-round opponent reads are simpler when opponents are data anyway
  ([ADR 0006](0006-a-run-against-a-stream.md)).

## Considered and rejected

- **Pure data with no modifier stage.** Simplest, but rule-modifier Units become
  impossible — not merely awkward — and they're among the most interesting cards in the
  genre.
- **A Rust function per Unit.** Unlimited expressiveness, but contradicts the
  data-centric requirement outright: every new Unit would need a recompile.
- **Defer the hook until a Unit demands it.** Ordinarily right — one adapter means a
  hypothetical seam. Rejected because the survey converts the hypothetical into a
  measured ~30%, and because this particular seam is much cheaper to build in than to
  retrofit.
