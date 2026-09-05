# Battlegrounds Demake

A minimalist demake of Hearthstone: Battlegrounds — a vertical slice with five deliberate
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

**Beat**: One Slot's resolution as the sweep moves left to right; the facing Units
resolve *synchronously*.
_Avoid_: tick, turn, step, exchange

## Units and Parties

**Unit**: A thing standing in a Slot, with its own stats and state. Two Units of one
Definition are independent.
_Avoid_: minion, creature, entity, character

**Unit Definition**: The immutable *kind* description — stats, tribes, keywords,
abilities — as written in a data file.
_Avoid_: card, template, blueprint, prototype

**Card**: A Unit's visual form — a 1:1.6 rectangle, presentation only. No separate token
class, unlike Battlegrounds.

**Party**: The Units a Player brings to the Action Phase, in their Slots. Ordering is
the central decision.
_Avoid_: board, warband, army, lineup, team, roster

**Slot**: One of **8** ordered positions a Party occupies — the Action Phase's
resolution order. May be empty.
_Avoid_: position, index, tile

**Board**: Both Parties' Slots, facing each other; Slot *i* faces Slot *i*.

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
