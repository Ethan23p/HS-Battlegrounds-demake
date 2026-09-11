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

Given directly:

- Data-oriented, highly configurable
- Deep back-end, shallow front-end
- Rust back-end
- Seeded RNG

**Q: Is asynchronous play still the model?**

> design decision: gameplay is offline, single player by obligation; if I get to develop
> this into a full product, it will probably be distantly similar to Battlegrounds, more
> like Super Auto Pets - non-synchronous matches, that's as far as it's worth figuring
> out ahead of time;

**Q: How does a unit choose its target within a beat?**

> Random targeting, like Battlegrounds — one attacker, one random defender per side,
> unless taunt is in play. If a taunt unit is in play on the opposing team, it always
> gets targeted instead.
>
> This isn't a feature/departure, just makes it in for clarification.

#### Beats

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

#### Death-resolution timing

**Q: When a unit dies mid-beat, when does its death actually take effect?**

> a unit doesn't die within a beat - death is triggered during the nextmost beat after a
> unit reaches 0 health. For example:
> Beat A
> unit Charlie receives 2 damage (goes from 2 to 0 health)
> Beat B
> unit Charlie dies, triggering their 'on death'

#### Keywords

**Q: Which keywords are in, and does any of them work differently than it does in Battlegrounds?**

> Same as Battlegrounds — Taunt, Divine Shield, Poisonous, Windfury, Reborn, and Rally
> (Rally is from the latest season, it's basically 'on attack') all unchanged

#### Abilities-as-data

**Q: How should a card's abilities be represented — what does defining a new one look like?**

> not cards but units - keep it simple, data-driven; more complex than that will have to
> be a dedicated discussion

#### Board size & anchoring

Given directly:

> the typical board is 8 slots;
> board logic anchors on the left-most position;
> I'd like to concept out some sort of internal physics - possibly with conservation of
> energy such that "going infinite" is far less feasible than in Battlegrounds. I haven't
> decided on the details, but they shouldn't be blocking yet;

*Asked whether an earlier sketch of three correlated resources was the same idea:*

> the resources discussion was the same thread of thought, but I'd rather discard that
> and leave it at what I stated in the previous message

### Ongoing

In the order the decisions were made. Context lines are Claude's, and only there to carry
the full implications of the quotation; the quotations are Ethan's.

#### The board is the environment

*Claude asked whether to rename `Board` and `Party` in the code, having noticed that
"Board size & anchoring" says "the typical board is 8 slots" while the code used
`Board` for both sides at once and `Party` for one side's eight slots.*
([0005](transcripts/0005-the-fresh-start-and-the-intent-clock.md))

> a board is the environment, with two opposing parties on it, parties consisting of
> units, units inhabiting slots on the board.

#### Beats are the only measure of time

*The engine had been walking slot indices 1..8 and calling each lap a "pass", so a pass
was a second clock alongside beats.*
([0005](transcripts/0005-the-fresh-start-and-the-intent-clock.md))

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
([0005](transcripts/0005-the-fresh-start-and-the-intent-clock.md))

> nice, all of that sounds more clean. Actions was the previous model - but, indeed,
> instead of actions I prefer intent

#### Beats slice time; interactions are concurrent within them

*The engine had both sides acting in the same beat, which nothing on record established
— Battlegrounds alternates, and gives one side the first swing on a coin flip. Claude
asked whether a beat belongs to one side or to both.*
([0006](transcripts/0006-the-concurrent-beat.md))

> Regarding beats - the missing piece is that I don't want there to be an ordering
> advantage (in Battlegrounds, it's a coin flip); another way of saying this is I want
> opposing side's actions to resolve on a regular basis, within the same beat when valid.
>
> Your instinct will be to complicate this, but I have the solution for you: Battlegrounds
> uses a non-concurrent transaction model (all steps **and** interactions have an origin
> and a target) but this app should use a concurrent transaction model (interactions have
> an origin and a target, beats are just how time is sliced).
>
> Another way to say it: in Battlegrounds, steps/interactions resolve rapidly and
> logically (the next troop doesn't attack until the steps caused by the previous troop
> are resolved); this app is **identical** but instead of steps/interactions this app has
> beats and interactions, distinct.

*Restated as an execution model:*

> Battleground's model is like a single threaded simulation which could theoretically get
> blocked by a single tick which is growing infinitely, this model is like a proper
> multi-threaded simulation. Does that make sense? Interactions/steps in Battlegrounds are
> strictly stacked.

*On why beats exist at all, rather than letting everything act at once:*

> Aside from the logic, the reason I declare beats is that I still think it's valuable for
> there to be a cohesive narrative which is progressing along, relatively singularly. else
> the default would seem to be each unit attacking at the same time, which is not desired.

#### A Divine Shield absorbs its beat

*The first consequence of the above. A shield was being spent on whichever blow the engine
reached first, and the engine always walked the player side first, so a shielded unit
fared differently depending on which side of the board it stood on — a mirrored board
resolved 160 against 167. Claude proposed that a shield instead absorb everything its beat
brings and break at the top of the next, the same shape as death-resolution timing.*
([0006](transcripts/0006-the-concurrent-beat.md))

> yes, that sounds very valid; approved on 1 & 2.
