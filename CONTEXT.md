# Battlegrounds Demake

A deterministic auto-battler engine: a minimalist core of Hearthstone: Battlegrounds'
gameplay, with several rules deliberately changed.

This file is a **glossary and nothing else** — not a spec, not a design doc. Its job is
to make us use one word per concept. A term lands here once a decision has actually been
made; if a word we throw around in conversation isn't here, it isn't settled yet.

Where Ethan has a word for something, that word wins. Terms marked **(provisional)** are
Claude's proposals awaiting his ratification.

## Language

### Structure of a match

**Match**:
A complete contest between Seats, played as a sequence of Rounds until one Seat wins.
_Avoid_: game, lobby, session

**Run**:
One playthrough, from a Match's start to its end. Currently coextensive with a Match; the
word exists because resources are scoped **in-run**, and a *meta* scope outside the Run is
anticipated beyond the prototype.

**Seat**:
A participant position in a Match, occupied by a human or a bot. The engine cannot tell
the difference and must never need to.
_Avoid_: player, agent, opponent, user

**Round**:
One Prep Phase followed by one Action Phase.
_Avoid_: turn

**Prep Phase**:
The half of a Round in which a Seat spends resources — buying, selling, rerolling, and
arranging its Board. The only half that accepts input. Unbounded in time: it ends when
the Seat says it ends, and ending it is what begins the Action Phase.
_Avoid_: recruit phase, tavern phase, shop phase, buy phase, turn

**Action Phase**:
The half of a Round in which two Boards fight. Accepts no input: fully determined by the
two Boards and the Seed. Named for the Minions, which act; the Seat does not.
_Avoid_: combat, battle, fight

**Beat**:
The unit in which the Action Phase advances — one moment of the simulation. Everything
resolving within a Beat resolves simultaneously, against the state at the Beat's opening;
Beats themselves are strictly ordered. A Beat is a slice of a simulation playing out, not
a turn anyone takes.
_Avoid_: tick, turn, step, round, exchange, action

**Command** *(provisional)*:
A single instruction a Seat issues during its Prep Phase — buy, sell, reroll, freeze,
reposition, tier up, end. The only input the engine accepts. Distinct from anything a
Minion does: Seats command, Minions act.
_Avoid_: action, move, input, play

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
The ordered row of Minions a Seat brings to the Action Phase. Position is meaningful.
_Avoid_: warband, army, field, lineup, team

**Token**:
A Minion Definition that can only enter play by being summoned by something else. Never
appears in a Shop.

**Tribe**:
A family a Minion Definition belongs to, which abilities can key off.
_Avoid_: type, race, family, class

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
A single declared change — buff, summon, damage, grant. Effects are what Abilities do;
Commands are what Seats issue. They are not the same thing and never share a name.
_Avoid_: action, outcome, result

**Selector**:
The description of which Minions an Effect lands on. A Selector *produces* targets; it
is not itself a target.
_Avoid_: target, filter, query

### Resources

The three axes are deliberately **separate but loosely correlated** — convertible, but
never freely or quickly. All three are scoped in-run. None is strictly conserved; each is
**bounded**, with large gains taxed and losses limited.

**Economy** *(definition provisional)*:
The spending resource. What a Seat pays with in its Prep Phase.
_Avoid_: gold, money, coins

**Power** *(definition provisional)*:
The stat resource — the attack and health carried by Minions on a Board.
_Avoid_: stats, strength

**Minions** *as a resource* — **UNRESOLVED, NAME COLLIDES**:
The third axis: bodies, board presence, count. The word collides with **Minion**, the
entity, and needs a distinct name before it enters this glossary properly.

**Conservation of values**:
The principle that the three resources move between places rather than appearing from
nothing. Bounded rather than strict — see [ADR 0005](docs/adr/0005-three-bounded-resources.md).

### The Shop and the Pools

**Shop**:
The Minions offered for purchase to one Seat during its Prep Phase.
_Avoid_: tavern, offers, store, market

**Tavern Tier**:
A Seat's level, which bounds the Minion Definitions its Shop may offer.
_Avoid_: level, rank, tech

**Minion Pool**:
The finite multiset of Minions available to be offered in Shops. Buying removes from it;
selling returns to it.
_Avoid_: deck, bag, supply, library, bare "pool"

**Opponent Pool**:
The set of Boards a Seat may be matched against. An opponent is drawn from it in advance
of the Prep Phase, not at the moment the Action Phase begins.
_Avoid_: bare "pool", ladder, queue

**Seed**:
The single number that, together with the Seats' Commands, determines every random
outcome in a Match. A Match is reproducible from its Seed and its Commands, indefinitely.
