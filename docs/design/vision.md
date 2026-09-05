# Vision

> A minimalist demake of Hearthstone: Battlegrounds with a number of key tweaks that make
> it a distinct auto-battler.

A gameplay vertical slice of Battlegrounds, *but with* the deltas below. The deltas are
the project; everything else is Battlegrounds.

## The default rule

**Where we have not deliberately changed something, it works however Battlegrounds works.**

This is load-bearing: an unanswered question usually already has one — go look at what
Battlegrounds does — and the delta list below is exhaustive, not indicative. Magnitudes
inherit too: numbers start at Battlegrounds' values, anchored on **1** as the atom.

The aesthetic goal, which outranks fidelity where the two conflict: **clean rules with
unambiguous resolution.** Given a Board, it should be possible to say what happens next
and be right, without knowing a hidden ordering. Where Battlegrounds resolves something by
an invisible coin flip or an emergent turn order, that's a candidate for replacement.

## What we keep

Full fidelity on mechanics, not content. Divine Shield, Poisonous, Reborn, Deathrattles,
Tribes, Tiers, and — importantly — cascading Triggers, where a death summons a Token whose
arrival fires a third Unit's Ability. The fiddly interactions are the point;
"minimalist" doesn't mean cutting them.

Content is minimal by contrast: a small original set of Unit Definitions, roughly one per
Effect, sized to demonstrate the vocabulary rather than fill a pool.

## The five deltas

### 1. A Run against a stream, not a lobby — **SETTLED**

See [ADR 0006](../adr/0006-a-run-against-a-stream.md), superseding
[ADR 0002](../adr/0002-two-seats-not-eight.md). One Player faces a stream of opposing
Parties drawn from a pool. Battlegrounds' eight-player free-for-all and its lobby are
gone, and the two-Seat halfway house went with them: once delta 5 made the Prep Phase
unbounded and the opponent a Party drawn in advance, a second Seat had nothing left to do.

### 2. The Action Phase is a left-to-right sweep of Beats — **SETTLED**

See [ADR 0003](../adr/0003-the-action-phase-is-a-simulation-of-beats.md). The Action
Phase sweeps Slot by Slot, left to right; the two Units facing each other in a Slot
resolve synchronously. Each such moment is a Beat.

The framing came first: an auto-battler's combat is a *simulation playing out*, and
discrete per-unit actions are a holdover from card games and tabletop. The sweep keeps a
cohesive narrative while removing every targeting decision — and with it, all
target-selection randomness. Ordering the Party becomes the central skill.

Two keywords are casualties needing reinvention: **Taunt** has no target choice left to
constrain, and **Windfury** has no turn to take twice.

### 3. An intentional economy across three axes — **PRINCIPLE SETTLED, DYNAMICS DEFERRED**

See [ADR 0005](../adr/0005-three-bounded-resources.md). **Economy, Power, Units** —
separate, loosely correlated, convertible only slowly or at a cost. Scoped to a Run.

The delta is *intentionality*, not the economy itself — Battlegrounds already exchanges
currency for power, power for units, and units for meta-progress, emergently and
hand-balanced after the fact. We're designing that exchange instead.

Bounded rather than strictly conserved: damage in a single action is compressed, taxing
the big actor and cutting the small actor's losses, so magnitudes neither run away nor
collapse. Conversion paths and curve shapes are deferred until the game is running.

### 4. Offline-capable — **SETTLED, MOSTLY A CONSEQUENCE OF DELTA 5**

A complete Round must run with no network and no server. Asynchrony buys this: with the
opponent drawn in advance and its Party already known, there's nothing to wait for and
nobody to ask.

### 5. Asynchronous Rounds — **SETTLED**

See [ADR 0004](../adr/0004-asynchronous-matches.md). The Prep Phase is unbounded — it
ends when the Player ends it, which starts the Action Phase. The opposing Party is drawn
from a pool ahead of time rather than matched at the moment of the fight.

Two consequences: no Ability may ever consult a live opponent, since by fight time the
other side is data; and unbounded Prep gives up the timer as a design tool, so difficulty
must come from the position rather than the clock.

## Presentation

Visually in the neighbourhood of Battlegrounds, moving toward Marvel Snap. **Every
play-piece is a Card** — a 1:1.6 rectangle — with no separate token class, unlike
Battlegrounds. Cards are presentation only; the engine deals in Units and never mentions
them.

## What this is not

- Not a faithful Battlegrounds clone. Deltas 1-3 change the game materially.
- Not a content project. The card set demonstrates the vocabulary, not a collection.
- Not a live multiplayer game. One Player; opponents are data.

## A note on testing

Because the Unit set is original, we have **no external correctness oracle** for content
— nobody can tell us what our Units are supposed to do. The default rule helps: for
anything undeltered, Battlegrounds *is* the oracle. But where a delta bites, our rules
must be specified precisely enough for a test to assert against.
