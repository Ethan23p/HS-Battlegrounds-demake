# HS-Battlegrounds-demake — Design

## Process

This document is built by interview.

**Claude's role:** informational interviewer, tasked with asking an extremely minimal
question (the creation of which Ethan guides) which extracts from the User a sufficient
answer.

Before each of the following steps, Claude prints them verbatim in the chat to the User.

**Step 0**
Establish a file in the project root which contains sections for Vision, Departures, and
Design Decisions, if it doesn't exist.

**Step 1**
Choose a topic from: vision + each departure + each design decision; each topic should be
covered only once.
Then draft an extremely minimal question which straightforwardly prompts the User to
respond naturally and in detail about the topic.

**Step 2**
Assume that the User's response was sufficient.
Place only the question and the User's response, formatted nicely, in the document.

Repeat.

---

## Vision

**Q: What are you making, and why that?**

> I'd like to make a minimal demake of Hearthstone: Battlegrounds - just a gameplay
> vertical slice - with a handful of departures which should end up feeling like a
> fairly distinct autobattler game.

---

## Design Decisions

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

---

## Departures

**Order:** beats (done) → targeting (clarification, not a departure) → death-resolution
timing → keywords → abilities-as-data → board size & anchoring.

**Clarification (targeting — not a departure):**

**Q: How does a unit choose its target within a beat?**

> Random targeting, like Battlegrounds — one attacker, one random defender per side,
> unless taunt is in play. If a taunt unit is in play on the opposing team, it always
> gets targeted instead.
>
> This isn't a feature/departure, just makes it in for clarification.

### Departure 1 — beats

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

### Departure 2 — death-resolution timing

**Q: When a unit dies mid-beat, when does its death actually take effect?**

> a unit doesn't die within a beat - death is triggered during the nextmost beat after a
> unit reaches 0 health. For example:
> Beat A
> unit Charlie receives 2 damage (goes from 2 to 0 health)
> Beat B
> unit Charlie dies, triggering their 'on death'

### Departure 3 — keywords

**Q: Which keywords are in, and does any of them work differently than it does in Battlegrounds?**

> Same as Battlegrounds — Taunt, Divine Shield, Poisonous, Windfury, Reborn, and Rally
> (Rally is from the latest season, it's basically 'on attack') all unchanged

### Departure 4 — abilities-as-data

**Q: How should a card's abilities be represented — what does defining a new one look like?**

> not cards but units - keep it simple, data-driven; more complex than that will have to
> be a dedicated discussion

### Departure 5 — board size & anchoring

Given directly:

> the typical board is 8 slots;
> board logic anchors on the left-most position;
> I'd like to concept out some sort of internal physics - possibly with conservation of
> energy such that "going infinite" is far less feasible than in Battlegrounds. I haven't
> decided on the details, but they shouldn't be blocking yet;

**Q: Earlier you described three correlated resources (economy, power, units) with bounded conservation. Is that the same thing as the "internal physics / conservation of energy" idea you just mentioned, or are they separate?**

> the resources discussion was the same thread of thought, but I'd rather discard that
> and leave it at what I stated in the previous message
