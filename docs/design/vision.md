# Vision

> A minimalist core of the Hearthstone: Battlegrounds gameplay — the most interesting
> version that is still genuinely doable.

Battlegrounds is the reference, not the target. We are keeping the shape of the game
(buy minions in a Recruit Phase, watch them fight in a Combat, repeat) and the fidelity
of its rules interactions, while deliberately changing five things about it.

The aesthetic goal, which outranks fidelity wherever the two conflict: **clean rules with
unambiguous resolution.** Given a board state, it should be possible to say what happens
next and be right, without knowing a hidden ordering. Where Battlegrounds resolves
something by an invisible coin flip or an emergent turn order, that is a candidate for
replacement rather than reimplementation.

## What we keep

Full fidelity on mechanics, not on content. Taunt, Divine Shield, Poisonous, Windfury,
Reborn, Deathrattles, Tribes, Tavern Tiers, and — importantly — cascading Triggers, where
a death summons a Token whose arrival fires a third Minion's Ability. The fiddly
interactions are the point; they are what makes the engine worth building and what
["minimalist"](#) must not be taken to mean cutting.

Content is minimal by contrast: a small custom set of Minion Definitions, roughly one per
Effect, sized to demonstrate the vocabulary rather than to fill a card pool. See
[Card set](#card-set) below.

## The five deltas

### 1. One-on-one, not an eight-player free-for-all — **SETTLED**

See [ADR 0002](../adr/0002-two-seats-not-eight.md). Two Seats, no lobby, no matchmaking,
no elimination order. Note that delta 5 puts pressure on what "two Seats" means.

### 2. The Action Phase is a simulation of Beats — **SETTLED IN FRAMING, RATE OPEN**

See [ADR 0003](../adr/0003-the-action-phase-is-a-simulation-of-beats.md). Beats are
strictly ordered; everything inside one resolves simultaneously. The narrative is
sequential, the resolution is not — you watch it unfold, and nobody goes first.

The framing is the decision: an auto-battler's combat is a *simulation playing out*, and
carving it into discrete per-minion actions is a holdover from card games and tabletop,
not something the form demands.

Still open, and gating v0.1: whether every Minion acts every Beat, or whether Minions have
rates and a Beat advances a clock. Also unresolved downstream of that — what Taunt
constrains when nobody chooses a target, and what Windfury doubles when there is no turn.

### 3. Conservation of values across three axes — **PRINCIPLE SETTLED, MECHANISM OPEN**

See [ADR 0005](../adr/0005-three-bounded-resources.md). **Economy, Power, and Minions**:
technically separate, loosely correlated, convertible only slowly or at a cost. Scoped
in-run. Bounded rather than strictly conserved — large gains taxed, losses limited.

The payoff is that three loosely-coupled axes give three genuinely different kinds of
problem ("rich but weak" is not the same trouble as "strong but out of bodies"), and that
a damping curve bounds scaling structurally instead of through per-card balance patches.

Undecided: what each axis precisely is, what the conversion paths cost, the shape of the
tax and loss-limit curves, and what to call the third axis given that "Minions" already
names the entity.

### 4. Offline-capable — **SETTLED, AND MOSTLY A CONSEQUENCE OF DELTA 5**

A complete Round must run with no network and no server. Asynchrony is what buys this:
with the opponent drawn in advance and its Board already known, there is nothing to wait
for and nobody to ask.

### 5. Asynchronous matches — **SETTLED**

See [ADR 0004](../adr/0004-asynchronous-matches.md). The Prep Phase is unbounded — it ends
when the Seat ends it, and ending it is what starts the Action Phase. The opponent is drawn
from an Opponent Pool ahead of time rather than matched at the moment of the fight.

Two consequences worth holding onto: no Ability may ever consult a live opponent, because
by fight time the other side is data; and unbounded Prep gives up the timer as a design
tool, so difficulty has to come from the position rather than the clock.

## Card set

Original Minion Definitions, not Battlegrounds' — a deliberately small set built to
exercise each Effect in the vocabulary roughly once, in the spirit of a demake's "box art"
done with tokens. This means the card set doubles as the engine's test surface: if an
Effect exists in the vocabulary, some Minion demonstrates it, and if no Minion needs an
Effect, that Effect should not exist.

Note the consequence for testing: unlike a faithful reimplementation, we have **no
external correctness oracle**. Nobody can tell us what our Minions are supposed to do. Our
rules have to be self-consistent and specified precisely enough that a test can assert
against them.

## What this is not

- Not a faithful Battlegrounds clone. Deltas 1-3 change the game materially.
- Not a content project. The card set is a demonstration surface, not a collection.
- Not a graphics project. See [ADR 0002](../adr/0002-two-seats-not-eight.md) for the seat
  model and the roadmap for the frontend's intended thinness.
