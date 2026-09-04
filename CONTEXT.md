# Battlegrounds Demake

A minimalist demake of Hearthstone: Battlegrounds with a number of key tweaks that make it
a distinct auto-battler.

This file is a **glossary and nothing else** — not a spec, not a design doc. Its job is
to make us use one word per concept. A term lands here once a decision has actually been
made; if a word we throw around in conversation isn't here, it isn't settled yet.

Where Ethan has a word for something, that word wins. Terms marked **(provisional)** are
Claude's proposals awaiting his ratification.

**Deliberately left non-specific:** *pool* and *action*. There can be a pool of opponents,
of units, of effects, of currency; a player takes actions and so does a unit. Both words
stay generic and get qualified in context rather than being reserved.

## Language

### Structure of play

**Run**:
One playthrough, from start to end. The Player survives a stream of opposing Parties. All
resources are scoped to a Run; a *meta* scope outside it is anticipated beyond the
prototype.
_Avoid_: game, match, session

**Player**:
The one who builds a Party and issues actions. There is exactly one; opponents are data,
not participants.
_Avoid_: seat, user, agent

**Round**:
One Prep Phase followed by one Action Phase, against one opposing Party.
_Avoid_: turn

**Prep Phase**:
The half of a Round in which the Player spends resources — buying, selling, rerolling, and
arranging the Party. The only half that accepts input. Unbounded in time: it ends when the
Player ends it, and ending it is what begins the Action Phase.
_Avoid_: recruit phase, tavern phase, shop phase, buy phase, turn

**Action Phase**:
The half of a Round in which two Parties fight. Accepts no input: fully determined by the
two Parties and the Seed.
_Avoid_: combat, battle, fight

**Beat**:
The unit in which the Action Phase advances — one moment of the simulation, resolving one
Slot. The Action Phase sweeps Slot by Slot from left to right; the two Units facing each
other in a Slot resolve *synchronously*. A Beat is a slice of a simulation playing out,
not a turn anyone takes.
_Avoid_: tick, turn, step, exchange

### Units and Parties

**Unit**:
A single thing standing in a Slot, with its own current stats and state. The general term
for a play-piece. Two Units of the same Unit Definition are wholly independent.
_Avoid_: minion, creature, entity, character

**Unit Definition**:
The immutable description of a *kind* of Unit — base stats, tribes, keywords, abilities —
as written in a data file.
_Avoid_: card, template, blueprint, prototype

**Card**:
The visual representation of a Unit — a 1:1.6 rectangle. Every play-piece is a Card, with
no separate class of token piece; this is a deliberate improvement on Battlegrounds, where
tokens are visually a different kind of object. A presentation concept: the engine deals in
Units and never mentions Cards.

**Party**:
The Units a Player brings to the Action Phase, in the Slots they occupy. Ordering is the
Party's central design decision.
_Avoid_: board, warband, army, lineup, team, roster

**Slot**:
One of **8** positions a Party's Units occupy. Slots are ordered left to right, and that
order is the order the Action Phase resolves in. A Slot may be empty.
_Avoid_: position, index, tile

**Board**:
The playing surface: both Parties' Slots, facing each other. Slot *i* of one Party faces
Slot *i* of the other.

**Token**:
A Unit Definition that can only enter play by being summoned by something else. Never
appears in a Shop. Visually still a Card like any other.

**Tribe**:
A family a Unit Definition belongs to, which abilities can key off.
_Avoid_: type, race, class

**Keyword**:
A persistent property the Action Phase consults directly rather than executing — Taunt,
Divine Shield, Poisonous, Windfury, Reborn.
_Avoid_: flag, status, trait, buff

### Abilities

**Ability**:
One Trigger, an optional condition, and an ordered list of Effects.

**Trigger**:
The moment at which an Ability fires.
_Avoid_: event, hook, listener, callback

**Effect**:
A single declared change — buff, summon, damage, grant.
_Avoid_: outcome, result

**Selector**:
The description of which Units an Effect lands on. A Selector *produces* targets; it is
not itself a target.
_Avoid_: target, filter, query

### Resources

Three axes — **Economy, Power, Units** — technically separate, loosely correlated,
convertible only slowly or at a cost. Scoped to a Run. Magnitudes follow Battlegrounds'
until there is a reason to differ, anchored on **1** as the atom: the most basic,
unexceptional unit of measure.

**Economy**:
The spending resource. What the Player pays with during a Prep Phase.
_Avoid_: gold, money, coins

**Power**:
The stat resource — attack and health carried by Units.
_Avoid_: stats, strength

**Units** *as a resource*:
The discrete axis: Units are the granular quantity out of which a Party is constructed.
Distinguished from **Unit**, the individual play-piece, by context alone.

**Conservation of values**:
The principle that the three resources move between places rather than appearing from
nothing. Bounded rather than strict — see
[ADR 0005](docs/adr/0005-three-bounded-resources.md).

### The Shop

**Shop**:
The Units offered for purchase to the Player during a Prep Phase.
_Avoid_: tavern, store, market

**Tier**:
The Player's level, which bounds the Unit Definitions the Shop may offer.
_Avoid_: tavern tier, level, rank, tech

**Seed**:
The single number that, together with the Player's actions, determines every random
outcome in a Run. A Run is reproducible from its Seed and its actions, indefinitely.
