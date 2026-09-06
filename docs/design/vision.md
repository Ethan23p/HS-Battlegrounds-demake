# Vision

> A minimalist demake of Hearthstone: Battlegrounds with a number of key tweaks that make
> it a distinct auto-battler.

A gameplay vertical slice of Battlegrounds, *but with* the nine deltas below. The deltas
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

## The nine deltas

The list is exhaustive: anything not here works however Battlegrounds works. Each delta
cites the message where Ethan introduced it, and the later ones that shaped it. Quotes
are verbatim, and the quote is the locator: transcript anchors move when a session is
re-exported, but his words don't.

### The shape of a Run

#### 1. A Run against a stream, not a lobby — SETTLED

[ADR 0006](../adr/0006-a-run-against-a-stream.md), superseding
[ADR 0002](../adr/0002-two-seats-not-eight.md). One Player faces a stream of opposing
Parties drawn from a pool — no eight-player free-for-all, no lobby, no second Seat.

> *instead of an 8 player FFA, I'd like to start with a simplified 1v1 model* —
> [transcript 0001](../transcripts/0001-project-kickoff.md). Reduced the rest of the way
> in [0002](../transcripts/0002-vocabulary-and-action-phase.md): *indeed, I think we
> should think of it as a stream of opposing parties.*

#### 2. Asynchronous Rounds — SETTLED

[ADR 0004](../adr/0004-asynchronous-matches.md). The Prep Phase is unbounded, ending when
the Player ends it; the opposing Party is drawn from a pool ahead of time. No Ability may
ever consult a live opponent, and difficulty must come from the position, not a clock.

> *instead of being synchronous match-ups I'd like them to be asynchronous in a
> particular way* — [transcript 0001](../transcripts/0001-project-kickoff.md).
> Specified in [0002](../transcripts/0002-vocabulary-and-action-phase.md): *asynchronous
> matches meaning unbounded prep phase, opponent is pre-selected from a pool randomly,
> action phase begins for a given player in response to ending their prep phase.*

#### 3. Offline-capable — SETTLED, and mostly a consequence of delta 2

A complete Round runs with no network and no server: with the opponent drawn in advance
and known, there's nothing to wait for.

> *instead of being always-online I'd like the capacity for offline play* —
> [transcript 0001](../transcripts/0001-project-kickoff.md).

### How a fight resolves

#### 4. The Action Phase advances in Beats, and both sides act in the same one — SETTLED

[ADR 0003](../adr/0003-the-action-phase-is-a-simulation-of-beats.md),
[ADR 0008](../adr/0008-targeting-is-random-simultaneity-is-the-only-delta.md),
[ADR 0010](../adr/0010-the-clock-is-a-beat-counter.md). A Beat is a time-step of the
Board: in each one, the left-most Unit of each Party that still owes a turn acts — on both
sides at once, so nobody swings first. Beats count from 1 and never reset; there is no
unit of time above them. Targeting stays random and Taunt still constrains it, exactly as
in Battlegrounds. The one real consequence: two evenly-matched Units can now trade blows
and die together.

> *instead of resolving combat back and forth in turns, I'd like each turn of attacks to
> resolve simultaneously (rock, paper, scissors is a real model)* —
> [transcript 0001](../transcripts/0001-project-kickoff.md). Reframed away from discrete
> actions in [0002](../transcripts/0002-vocabulary-and-action-phase.md): *Instead of
> proceeding in discrete 'actions' this app should proceed in 'beats'*, then given its
> shape — *the action phase resolves from left to right and units in the same slot are
> synchronous.* Made a clock in
> [0003](../transcripts/0003-action-phase-corrections.md): *let's make canonical that a
> 'beat' is a time-step - so no per party passes.* Made the *only* clock in
> [0004](../transcripts/0004-the-beat-counter.md): *the passage of time only happens
> through beats, beats progress at a consistent rate in one direction within a round(they
> don't reset), there's no concept of "now we are looping back to the start" but there
> should be a concept of "we've ennumerated through all party members"*

#### 5. The attacker dies last — SETTLED

[ADR 0008](../adr/0008-targeting-is-random-simultaneity-is-the-only-delta.md).
Simultaneity opens a question Battlegrounds never has to answer, because its resolution
is sequential: when a trade kills both, whose death resolves first. The attacker's
resolves last, so a kill stays attributable to the Unit that landed it even when that
Unit is dying too.

> *Demake, damage/death resolution? Simultaneous, attacker dies last. (This last one was
> an ambiguity I hadn't anticipated)* —
> [transcript 0003](../transcripts/0003-action-phase-corrections.md).

### The Board

#### 6. Eight Slots to a Party — SETTLED

Battlegrounds seats seven. Eight is a power of two, and a Party built to fill it makes
every empty Slot a decision.

> *I'll make the call that a full 'party' of units should occupy 8 slots on the board*
> ... *(I just like 8 &/or powers of 2)* —
> [transcript 0002](../transcripts/0002-vocabulary-and-action-phase.md).

#### 7. The Party is left-anchored, and never holds a gap — SETTLED

[ADR 0009](../adr/0009-the-party-is-left-anchored.md), whose mechanism
[ADR 0010](../adr/0010-the-clock-is-a-beat-counter.md) replaces. Hearthstone keeps a board
centred and Battlegrounds is left-anchored only by convention, so neither lets you say
from the board alone who acts next. Here a Party is an unbroken run anchored on its
left-most Unit: a death closes it up at once, and a Slot is simply where a Unit stands
rather than an address that can be empty. Nothing is disturbed by that closing up, because
turn order is each Unit's own readiness and not a count of Slots.

> *Instead of Battleground's ambiguity about positioning, this app will anchor the party
> on the left-most unit and occasionally compact toward them. Compaction is persistently
> applied in the prep phase, then applied scarcely in the action phase* —
> [transcript 0003](../transcripts/0003-action-phase-corrections.md). Made continuous, and
> made a property of the data rather than a scheduled operation, in
> [0004](../transcripts/0004-the-beat-counter.md): *instead of focusing on absolute values
> (SLOT-3 = EMPTY) let's place things relative to each other (the third unit died)*.

#### 8. Every play-piece is a Card — SETTLED

A 1:1.6 rectangle, with no separate class of token piece. Presentation only: the engine
deals in Units and never mentions Cards.

> *I'm settled on card; cards represent units, the physical representation of units will
> largely be cards, 1:1.6 cards. This is a nice improvement over Battlegrounds having
> 'tokens' randomly as play-pieces* —
> [transcript 0002](../transcripts/0002-vocabulary-and-action-phase.md).

### Resources

#### 9. An intentional economy across three axes — PRINCIPLE SETTLED, DYNAMICS DEFERRED

[ADR 0005](../adr/0005-three-bounded-resources.md). **Economy, Power, Units** —
separate, loosely correlated, convertible only slowly or at a cost, scoped to a Run. The
delta is *intentionality*: Battlegrounds already runs this economy emergently; we're
designing it instead. Bounded, not strictly conserved — damage in a single action is
compressed, taxing the big actor and cutting the small actor's losses.

> *I'd like to add a constraint in the form of "conservation of values", like in
> currency, possibly in stat gains, possibly in minion generation* —
> [transcript 0001](../transcripts/0001-project-kickoff.md). Shaped in
> [0002](../transcripts/0002-vocabulary-and-action-phase.md): *I adore resource
> management and I think the most interesting relationship for resources is to be
> technically separate, loosely correlated, but still correlated*, and *Bounded: this
> is just intuition, but that feels right - tax big gains, limit losses.*

## What is not a delta

Kept here because each has, at some point, been mistaken for one.

- **The original Unit set** is a content decision, not a rule change — *the custom card
  set will be very limited, pretty much just a minimal set to demonstrate each effect*
  ([0001](../transcripts/0001-project-kickoff.md)).
- **No heroes** is scope for the prototype, not a rule — *Q12 cut heroes*
  ([0002](../transcripts/0002-vocabulary-and-action-phase.md)). Battlegrounds has them;
  so may v0.4.
- **Targeting, Taunt, Windfury, and never attacking a Player** are Battlegrounds' own
  rules. Each was at one point written down here as a delta and is not one; see
  [ADR 0008](../adr/0008-targeting-is-random-simultaneity-is-the-only-delta.md) for how
  that drift happened and what it cost.

## Presentation

Visually near Battlegrounds, moving toward Marvel Snap.

## What this is not

- Not a faithful clone — deltas 1, 4 and 9 change the game materially.
- Not a content project — the card set demonstrates the vocabulary.
- Not live multiplayer — one Player; opponents are data.

## A note on testing

The Unit set is original, so we have **no external correctness oracle** — nobody can tell
us what our Units are supposed to do. The default rule helps where nothing's changed;
where a delta bites, our rules must be precise enough for a test to assert against.
