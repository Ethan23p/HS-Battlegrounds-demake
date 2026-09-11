# HS-Battlegrounds-demake — Design

This document is just an organizational device for pointing to the source of truth, which
is actual quotations from Ethan within the transcripts. (And the initial contents of
DESIGN.md) The intention is not that every line of code is attributable, but that
explicit design decisions are immortalized and used as guidance.

---

## Vision

**Q: What are you making, and why that?**

> I'd like to make a minimal demake of Hearthstone: Battlegrounds - just a gameplay
> vertical slice - with a handful of departures which should end up feeling like a
> fairly distinct autobattler game.

---

## Design Decisions

### Initial

Given directly, not through the question loop:

- Data-oriented, highly configurable
- Deep back-end, shallow front-end
- Rust back-end
- Seeded RNG

**Q: You mentioned wanting matches to be asynchronous — offline play, an unbounded prep phase, opponent drawn at random from a pool when you end your turn. Is that still the model, and what does it look like end to end?**

> design decision: gameplay is offline, single player by obligation; if I get to develop
> this into a full product, it will probably be distantly similar to Battlegrounds, more
> like Super Auto Pets - non-synchronous matches, that's as far as it's worth figuring
> out ahead of time;

**Clarification (targeting — not a departure):**

**Q: How does a unit choose its target within a beat?**

> Random targeting, like Battlegrounds — one attacker, one random defender per side,
> unless taunt is in play. If a taunt unit is in play on the opposing team, it always
> gets targeted instead.
>
> This isn't a feature/departure, just makes it in for clarification.

#### Departure 1 — beats

**Q: What's the first departure from Battlegrounds you want to talk about — what is it, and how does it work?**

> In Battlegrounds, the combat phase proceeds one step at a time - a step is a container
> containing a single interaction, which could be effectively anything (a unit action, an
> effect, hero power) - and many interactions spawn steps which have to resolve in place
> before the next step can be taken. These steps are always synchronous transactions and
> usually involve 1 origin entity and 1 target entity - except when they don't.
>
> Instead of Battlegrounds' 'step' this app has beats which are just like steps except
> they support concurrent transactions between arbitrary entities - in a BG's step, an
> interaction has to resolve before the next can begin, but this app's beats contain any
> arbitrary number of interactions we choose; nonetheless, most beats involve one
> interaction.

#### Departure 2 — death-resolution timing

**Q: When a unit dies mid-beat, when does its death actually take effect?**

> a unit doesn't die within a beat - death is triggered during the nextmost beat after a
> unit reaches 0 health. For example:
> Beat A
> unit Charlie receives 2 damage (goes from 2 to 0 health)
> Beat B
> unit Charlie dies, triggering their 'on death'

#### Departure 3 — keywords

**Q: Which keywords are in, and does any of them work differently than it does in Battlegrounds?**

> Same as Battlegrounds — Taunt, Divine Shield, Poisonous, Windfury, Reborn, and Rally
> (Rally is from the latest season, it's basically 'on attack') all unchanged

#### Departure 4 — abilities-as-data

**Q: How should a card's abilities be represented — what does defining a new one look like?**

> not cards but units - keep it simple, data-driven; more complex than that will have to
> be a dedicated discussion

#### Departure 5 — board size & anchoring

Given directly:

> the typical board is 8 slots;
> board logic anchors on the left-most position;
> I'd like to concept out some sort of internal physics - possibly with conservation of
> energy such that "going infinite" is far less feasible than in Battlegrounds. I haven't
> decided on the details, but they shouldn't be blocking yet;

**Q: Earlier you described three correlated resources (economy, power, units) with bounded conservation. Is that the same thing as the "internal physics / conservation of energy" idea you just mentioned, or are they separate?**

> the resources discussion was the same thread of thought, but I'd rather discard that
> and leave it at what I stated in the previous message

### Ongoing

In the order the decisions were made. Context lines are Claude's, and only there to carry
the full implications of the quotation; the quotations are Ethan's.

#### The board is the environment

*Claude asked whether to rename `Board` and `Party` in the code, having noticed that
Departure 5 says "the typical board is 8 slots" while the code used `Board` for both
sides at once and `Party` for one side's eight slots.*
([0004](transcripts/0004-the-fresh-start-and-the-intent-clock.md))

> a board is the environment, with two opposing parties on it, parties consisting of
> units, units inhabiting slots on the board.

#### Unclear phrasing gets cut, not defended

*Unprompted, on reading the comments in the adapted engine.*
([0004](transcripts/0004-the-fresh-start-and-the-intent-clock.md))

> I saw some head-scratcher phrasing in the code which would be better removed, still;
> for instance, something like "a beat without any action is not a beat" or something,
> if anything is at all unclear, might as well remove it or ask me about it;

#### Beats are the only measure of time

*The engine had been walking slot indices 1..8 and calling each lap a "pass", so a pass
was a second clock alongside beats.*
([0004](transcripts/0004-the-fresh-start-and-the-intent-clock.md))

> I don't like the concept of a 'pass'; I prefer there's only one variable representing
> time: beats. A pass is arbitrary - what matters is that beats are proceeding and units
> have intents;

*Asked in the same message, about what replaces the pass as the thing that tells the
engine when a cycle is over and when a party may re-anchor:*

> how do we know when to loop? when to compact? What's the data-oriented answer?

#### Intent

*Claude proposed that the answer to both questions is to put "whose beat is it" on the
unit rather than in a cursor over slots: a unit holds a count of how many more times it
means to act, the board renews them when nobody holds any, and compaction becomes safe
every beat because the count rides along when a unit slides. Claude asked whether to
call the field `actions` or `intent`.*
([0004](transcripts/0004-the-fresh-start-and-the-intent-clock.md))

> nice, all of that sounds more clean. Actions was the previous model - but, indeed,
> instead of actions I prefer intent
