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

#### The opponent pool is procedural, kept light

*Initial recorded that a random-draw-from-a-pool opponent was mentioned but never
confirmed, and the 0.3 roadmap entry named it a blocker: a pool of what — pre-built
parties, other runs, something generated? Asked directly once 0.1 shipped.*
([0007](transcripts/0007-the-front-end-and-the-scratchpad.md))

> Procedurally generated, not overly involved at this stage

#### Every in-game object draws from, and returns to, a scarce shared pool

*0.3 planning raised whether the shop's Unit pool should be scarce (as in real
Battlegrounds, where a bought copy is unavailable to be drawn again until sold or the
Unit dies) or effectively infinite, since this game is confirmed single-player. Ethan's
answer widens the question into a general principle rather than a yes/no on Units alone.*
([0010](transcripts/0010-scarcity-and-best-of-three.md))

> I'd like to account for scarcity, even at this early stage; I'm not thinking like per
> unit tracking, but basically all in game objects (abstractly speaking) take resources
> from a scarce pool and, in theory, end up with their resources back in that pool.
> Early on I referred to this as having an internal physics, like laws of conservation.

#### Runs are best of 3

*Damage-on-loss (what a losing player's health total should take, and from what) turned
out to depend on a broader meta-game structure that isn't decided yet. Asked directly
during 0.3 planning.*
([0010](transcripts/0010-scarcity-and-best-of-three.md))

> For "damage on loss" this depends on the broad meta game which we land on, which I'm
> not sure of, so I'd say runs are best of 3

#### Abilities are authored as data against a fixed vocabulary, extended on request -- not a general scripting layer

*0.4 planning asked how far to go on ability authoring: `units::Trigger`/`Condition`/
`Selector`/`Effect` already exist as an unexecuted, declarative vocabulary (Battlecry,
Deathrattle, Buff, Damage, Summon, and so on), but nothing runs it yet. The real fork was
whether to build the engine for that fixed vocabulary (extending it by hand as new
abilities need something it can't yet express) or to embed a general scripting layer
(e.g. Rhai) so any mechanic is expressible without an engine change. Ethan's answer:
build the engine for the fixed vocabulary, and treat Claude itself -- being asked to add
the one variant a new ability needs -- as the extensibility mechanism, rather than
building that flexibility into the runtime.*
([0011](transcripts/0011-mobile-playtest-and-ability-authoring.md))

> Yeah. See, sounds good. option c, that is. And part of the expectations that we can
> build upon is that, um, a little bit like a scripting layer is my access to you. So
> hypothetically, I can, uh, simply ask you to implement this or that, and then Voila. A
> bit later, I have that mechanic. So keep that in mind that maybe we are... we're not
> designing for, like, handwritten Abilities, nor are we preparing for necessarily a
> human compatible scripting, but we definitely want that flexibility for anything to be
> possible without breaking stuff. But I think, generally, you you build in a nice,
> maintainable way.

#### A fight's own changes to the party aren't permanent unless something specifies otherwise

*A playtest bug report: units that died in a fight weren't returning on the next round.
Claude's first fix attempt assumed real Hearthstone Battlegrounds makes death permanent
and treated a Deathrattle's Summon as an exception that should persist; Ethan corrected
both -- this is not a departure from Battlegrounds, and nothing in this engine currently
"specifies" any change as permanent, Deathrattle summons included.*
([0012](transcripts/0012-a-fights-changes-are-not-permanent.md))

> For clarification, the behavior I stated is how Battlegrounds works. In Battlegrounds
> the changes to the party within an action phase aren't permanent unless specified so.

> I noticed you said that the implementation matches Battlegrounds, but I'm asking for
> different behavior; and you said that units summoned by deathrattle should persist to
> the next round; neither are true/accurate. You're not implementing the mechanic as I
> said, which will just result in more work later on. I mean it precisely: In
> Battlegrounds the changes to the party within an action phase aren't permanent unless
> specified so.

#### Project vocabulary: Entity, Attribute, Trait, Passive, Trigger, Summon, Beat, Unit, Card, Intent

*Ethan laid out the project's own vocabulary in one pass, several terms replacing
Battlegrounds' own usage (Tribe -> Attribute, Keyword and Ability -> Trait, Token ->
Summon), others formalizing terms the code already used (Beat, Unit, Intent). Two points
needed a follow-up clarification, recorded here since the first pass over both was wrong:
whether "Passive" and a Trait tied to a Trigger are the only two shapes a Trait can take
(no -- a deliberately loose line, since something like a "dormant" Trait may not fit
either later), and what a persistent vs. temporary Summon distinguishes (not the
"specified permanent change" question the fight-revival fix just settled -- a persistent
Summon is one made during the Prep Phase, a temporary one during the Action Phase, and
the distinction isn't consequential yet).*
([0013](transcripts/0013-project-vocabulary.md))

> Entity: a distinct thing with an identity.
>
> Attribute: A static property of something. Use in place of 'tribe' in BG. More flexible
> than 'tribe' in Battlegrounds. (In ways that aren't consequential thus far)
>
> Trait: Any additional **behavior** beyond plain/vanilla behaviors. Use instead of
> keywords from BG. Use instead of abilities.
>
> Passive: A persistent behavior or persistent modification of a behavior.
>
> Trigger: A shared reference for 'point at which an effect comes into play.' Use instead
> of keyword as used in BG. Often prefixed by "on", such as "On death" or "on end of
> turn".
>
> Summon(persistent/temporary): A title for a unit which was summoned. Use instead of
> 'token' as used in BG.
>
> Pre-existing, worth formalizing:
>
> Beat: A single step in time. Best thought of in the context of a simulation; what
> happens within a beat is simultaneous; a beat has unbound capacity but if an action is
> dependent upon another thing, that action can't happen until the beat after the thing
> first came into play. Resolves ambiguity around synchronous events or transactions.
>
> Unit: an embodied entity. Visually represented by a card.
>
> Card: Visual representation of a unit. Often used interchangeably with unit. Use
> instead of 'troop' from BG.
>
> Intent: a general expression of intention by a given entity. Typically acted upon and
> exhausted during the next-most beat (dependent upon location), and non-cumulative.

> Trait, passive, trigger - Not quite, my original wording is the correct framing, I
> think. Passive traits is correct, then I don't think it'd be correct to say "triggered
> traits" but "traits with a trigger". It's just a semantic line in the sand for now -
> but I suspect it will be significant eventually due to unexpected cases like 'dormant
> traits' or something.
>
> summon - no, not my meaning. BG doesn't have the persistent/temporary distinction, but
> I think it could turn out to be useful so that's why I include it in the vocabulary. A
> persistent summon is like one that happens in the prep phase, a temporary summon is
> one that happens in the action phase, usually as part of an effect. It's not really
> consequential the majority of the time.
>
> How you address them in code is up to you, just try not to let implementation block my
> statements because I have a very clear mental model - I'm usually not suggesting
> anything legitimately logically inconsistent.
