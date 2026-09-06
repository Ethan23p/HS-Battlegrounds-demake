# Battlegrounds Demake

A minimalist demake of Hearthstone: Battlegrounds — a vertical slice with nine deliberate
rule changes (see [vision.md](docs/design/vision.md)).

This file is a **glossary and nothing else**. A term lands here once settled; a word we
use but don't list here isn't settled yet.

Where Ethan has a word, that word wins. **(provisional)** marks Claude's proposals
awaiting ratification. *Pool* and *action* stay deliberately generic, qualified in
context rather than reserved.

## Structure of play

**Run**: One playthrough, start to end; the Player survives a stream of opposing
Parties. Resources scope to it.
_Avoid_: game, match, session

**Player**: The one who builds a Party and acts. Exactly one; opponents are data.
_Avoid_: seat, user, agent

**Round**: One Prep Phase followed by one Action Phase, against one opposing Party.
_Avoid_: turn

**Prep Phase**: The input half of a Round — buy, sell, reroll, arrange. Unbounded;
ending it starts the Action Phase.
_Avoid_: recruit/tavern/shop/buy phase, turn

**Action Phase**: The half where two Parties fight, fully determined by the Parties and
Seed. No input.
_Avoid_: combat, battle, fight

**Beat**: One time-step of the Action Phase, belonging to the Board rather than to either
Party. In each Beat the left-most **Ready** Unit of each Party acts — on both sides
*simultaneously*, which is Battlegrounds' alternating turns merged into one. Beats count
from 1 and run one way for the whole Action Phase; nothing resets them, and there is no
unit of time above them. A Beat is also the span worth comparing across: the Board before
it and the Board after it differ by everything it did.
_Avoid_: tick, turn, step, exchange, pass

**Ready**: Of a Unit, still owing the clock a turn. Acting spends it. When neither Party
has a Ready Unit left, every Unit becomes Ready again — the moment *before the first Unit
acts*, and the only boundary the clock has (see
[ADR 0010](docs/adr/0010-the-clock-is-a-beat-counter.md)). So everything standing when the
Board came Ready has had its turn before any of it acts twice.
_Avoid_: active, awake, untapped, available

## Units and Parties

**Unit**: A thing standing in a Slot, with its own stats and state. Two Units of one
Definition are independent.
_Avoid_: minion, creature, entity, character

**Unit Definition**: The immutable *kind* description — stats, tribes, keywords,
abilities — as written in a data file.
_Avoid_: card, template, blueprint, prototype

**Card**: A Unit's visual form — a 1:1.6 rectangle, presentation only. No separate token
class, unlike Battlegrounds.

**Party**: The Units a Player brings to the Action Phase, in their Slots — an unbroken run
anchored on its left-most Unit, at most 8 long. Ordering is the central decision. A death
closes it up at once, so it never holds a gap.
_Avoid_: board, warband, army, lineup, team, roster

**Slot**: Where a Unit stands in its Party, counted **1 to 8** from the left. A Party of
three occupies Slots 1, 2 and 3. Not an address a Unit is assigned to and not something
that can be empty: a Party of three has three Slots.
_Avoid_: position, index, tile

**Board**: Both Parties, as they stand during the Action Phase. Targeting is random
(see [ADR 0008](docs/adr/0008-targeting-is-random-simultaneity-is-the-only-delta.md)),
not Slot-to-Slot.

**Token**: A Unit Definition that's summoned-only; never in a Shop.

**Tribe**: A family a Unit Definition belongs to, for abilities to key off.
_Avoid_: type, race, class

**Keyword**: A persistent property the Action Phase consults directly — Taunt, Divine
Shield, Poisonous, Windfury, Reborn.
_Avoid_: flag, status, trait, buff

## Abilities

**Ability**: One Trigger, an optional condition, an ordered list of Effects.

**Trigger**: The moment an Ability fires.
_Avoid_: event, hook, listener, callback

**Effect**: A single declared change — buff, summon, damage, grant.
_Avoid_: outcome, result

**Selector**: What Units an Effect lands on. Produces targets; isn't one itself.
_Avoid_: target, filter, query

## Resources

Three axes — **Economy, Power, Units** — separate, loosely correlated, convertible only
slowly or at a cost. Scoped to a Run, anchored on **1** as the atom.

**Economy**: The spending resource.
_Avoid_: gold, money, coins

**Power**: The stat resource — attack and health.
_Avoid_: stats, strength

**Units** *as a resource*: The discrete axis Units are counted on — distinct from
**Unit** by context alone.

**Conservation of values**: The three resources move rather than appear from nothing.
Bounded, not strict — see [ADR 0005](docs/adr/0005-three-bounded-resources.md).

## The Shop

**Shop**: The Units offered for purchase during a Prep Phase.
_Avoid_: tavern, store, market

**Tier**: The Player's level, bounding what the Shop may offer.
_Avoid_: tavern tier, level, rank, tech

**Seed**: The number that, with the Player's actions, determines every random outcome in
a Run — reproducible indefinitely.
