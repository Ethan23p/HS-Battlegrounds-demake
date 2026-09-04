# Battlegrounds Demake

A deterministic auto-battler engine: a minimalist core of Hearthstone: Battlegrounds'
gameplay, with several rules deliberately changed.

This file is a **glossary and nothing else** — not a spec, not a design doc. Its job is
to make us use one word per concept. A term lands here once a decision has actually been
made; if a word we throw around in conversation isn't here, it isn't settled yet.

## Language

### Structure of a match

**Match**:
A complete contest between Seats, played as a sequence of Rounds until one Seat wins.
_Avoid_: game, lobby, session, run

**Seat**:
A participant position in a Match, occupied by a human or a bot. The engine cannot tell
the difference and must never need to.
_Avoid_: player, agent, opponent, user

**Round**:
One Recruit Phase followed by one Combat.
_Avoid_: turn

**Recruit Phase**:
The half of a Round in which a Seat spends gold — buying, selling, rerolling, and
arranging its Board. The only half that accepts input.
_Avoid_: tavern phase, shop phase, buy phase, turn

**Combat**:
The half of a Round in which two Boards fight. Accepts no input: fully determined by the
two Boards and the Seed.
_Avoid_: battle, fight

**Tick**:
One simultaneous exchange of attacks inside a Combat. Every attack in a Tick is resolved
against the state at the Tick's start; deaths are applied at its end.
_Avoid_: turn, step, round, exchange

**Action**:
Something a Seat does during its Recruit Phase — buy, sell, reroll, freeze, reposition,
tier up, end. The only input the engine accepts.
_Avoid_: move, command, input, play

### Minions

**Minion Definition**:
The immutable description of a *kind* of minion — base stats, tribes, keywords,
abilities — as written in a data file.
_Avoid_: card, template, blueprint, prototype

**Minion**:
A single instance of a Minion Definition standing at a position on a Board, with its own
current stats and state. Two Minions of the same Definition are wholly independent.
_Avoid_: card, unit, creature, entity

**Board**:
The ordered row of Minions a Seat brings to Combat. Position is meaningful.
_Avoid_: warband, army, field, lineup, team

**Token**:
A Minion Definition that can only enter play by being summoned by something else. Never
appears in a Shop.

**Tribe**:
A family a Minion Definition belongs to, which abilities can key off.
_Avoid_: type, race, family, class

**Keyword**:
A persistent property that Combat consults directly rather than executing — Taunt,
Divine Shield, Poisonous, Windfury, Reborn.
_Avoid_: flag, status, trait, buff

### Abilities

**Ability**:
One Trigger, an optional condition, and an ordered list of Effects.

**Trigger**:
The moment at which an Ability fires.
_Avoid_: event, hook, listener, callback

**Effect**:
A single declared change — buff, summon, damage, grant. Effects are what Abilities do;
Actions are what Seats do. They are not the same thing and never share a name.
_Avoid_: action, outcome, result

**Selector**:
The description of which Minions an Effect lands on. A Selector *produces* targets; it
is not itself a target.
_Avoid_: target, filter, query

### Economy and randomness

**Shop**:
The Minions offered for purchase to one Seat during its Recruit Phase.
_Avoid_: tavern, offers, store, market

**Tavern Tier**:
A Seat's level, which bounds the Minion Definitions its Shop may offer.
_Avoid_: level, rank, tech

**Pool**:
The finite multiset of Minions available to be offered. Buying removes from it; selling
returns to it.
_Avoid_: deck, bag, supply, library

**Seed**:
The single number that, together with the Seats' Actions, determines every random
outcome in a Match. A Match is reproducible from its Seed and its Actions, indefinitely.
