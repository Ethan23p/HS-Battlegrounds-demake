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

See [ADR 0002](../adr/0002-two-seats-not-eight.md). Two Seats, no lobby, no
matchmaking, no elimination order.

### 2. Simultaneous Combat resolution — **DECIDED, MECHANISM OPEN**

See [ADR 0003](../adr/0003-simultaneous-combat-resolution.md). Attacks within a Tick
resolve against the Tick's opening state; deaths apply at its end. Rock-paper-scissors
rather than a turn order. This removes Combat's opening coin flip and makes each Tick a
pure, separately checkable function.

Still to design: how attackers pair with defenders when both sides choose at once, whether
every Minion attacks every Tick, what Taunt constrains under simultaneous choice, and what
Windfury means when there is no turn to take twice.

### 3. Conservation of value — **OPEN, AND THE MOST INTERESTING ONE**

The intent: value behaves like currency. It moves between places rather than appearing
from nothing. A buff's +2/+2 came from somewhere; a summoned Token's stats were paid for;
gold spent went somewhere rather than evaporating.

If this holds, several things follow for free: power growth is bounded without balance
patches, the Pool becomes a genuine contested resource rather than a probability
distribution, and every Effect becomes a *transfer*, which is both easier to reason about
and easier to render honestly to a player.

Undecided, and blocking: whether there is one conserved quantity or several (gold, stats,
bodies); whether conservation is global across both Seats and the Pool, or per-Seat;
whether it is strict or merely bounded; and what the exchange rate is between gold and
stats if they are the same substance.

### 4. Offline-capable — **DIRECTION SETTLED, IMPLICATIONS OPEN**

The engine must be able to run a complete Match with no network and no server. In
practice this is close to free given the architecture — the engine is a pure state machine
with no I/O — but it constrains what the asynchronous model in delta 5 is permitted to
assume.

### 5. Asynchronous match-ups — **OPEN, MEANING NOT YET PINNED DOWN**

Ethan has specified asynchronous "in a particular way", and that particular way has not
yet been described. Recorded here as an explicit hole rather than guessed at.

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
