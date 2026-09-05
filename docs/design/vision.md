# Vision

> A minimalist demake of Hearthstone: Battlegrounds with a number of key tweaks that make
> it a distinct auto-battler.

A gameplay vertical slice of Battlegrounds, *but with* the five deltas below. The deltas
are the project; everything else is Battlegrounds.

## The default rule

**Where we haven't deliberately changed something, it works however Battlegrounds
works.** An unanswered question usually already has one: go look at what Battlegrounds
does. Numbers start at Battlegrounds' values too, anchored on **1** as the atom.

The aesthetic goal, above fidelity where the two conflict: **clean rules with
unambiguous resolution** — given a Board, you should be able to say what happens next and
be right, without a hidden ordering. An invisible coin flip or emergent turn order is a
candidate for replacement.

## What we keep

Full fidelity on mechanics, not content: Divine Shield, Poisonous, Reborn, Deathrattles,
Tribes, Tiers, and cascading Triggers (a death summons a Token whose arrival fires a
third Unit's Ability). The fiddly interactions are the point. Content is minimal by
contrast — a small original Unit set, roughly one per Effect.

## The five deltas

### 1. A Run against a stream, not a lobby — SETTLED

[ADR 0006](../adr/0006-a-run-against-a-stream.md). One Player faces a stream of opposing
Parties drawn from a pool — no eight-player free-for-all, no lobby, no second Seat.

### 2. The Action Phase is a left-to-right sweep of Beats — SETTLED

[ADR 0003](../adr/0003-the-action-phase-is-a-simulation-of-beats.md). Slot by Slot, left
to right; the two Units facing each other in a Slot resolve synchronously — combat as *a
simulation playing out*, not discrete per-unit turns. Every targeting decision
disappears, and ordering the Party becomes the central skill. Casualties needing
reinvention: **Taunt** (no target choice left to constrain) and **Windfury** (no turn to
take twice).

### 3. An intentional economy across three axes — PRINCIPLE SETTLED, DYNAMICS DEFERRED

[ADR 0005](../adr/0005-three-bounded-resources.md). **Economy, Power, Units** —
separate, loosely correlated, convertible only slowly or at a cost, scoped to a Run. The
delta is *intentionality*: Battlegrounds already runs this economy emergently; we're
designing it instead. Bounded, not strictly conserved — damage in a single action is
compressed, taxing the big actor and cutting the small actor's losses. Conversion paths
and curve shapes come once the game is running.

### 4. Offline-capable — SETTLED, a consequence of delta 5

A complete Round runs with no network and no server: with the opponent drawn in advance
and known, there's nothing to wait for.

### 5. Asynchronous Rounds — SETTLED

[ADR 0004](../adr/0004-asynchronous-matches.md). The Prep Phase is unbounded, ending
when the Player ends it; the opposing Party is drawn from a pool ahead of time. No
Ability may ever consult a live opponent, and difficulty must come from the position, not
a clock.

## Presentation

Visually near Battlegrounds, moving toward Marvel Snap. **Every play-piece is a Card** —
a 1:1.6 rectangle, presentation only, with no separate token class.

## What this is not

- Not a faithful clone — deltas 1-3 change the game materially.
- Not a content project — the card set demonstrates the vocabulary.
- Not live multiplayer — one Player; opponents are data.

## A note on testing

The Unit set is original, so we have **no external correctness oracle** — nobody can tell
us what our Units are supposed to do. The default rule helps where nothing's changed;
where a delta bites, our rules must be precise enough for a test to assert against.
