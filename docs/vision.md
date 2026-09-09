# Vision

Two sections. The **vision statement** says what this project is. **Provenance** is the
evidence for it: Ethan's own words, in the order he said them.

The rule between them: no claim in the vision statement that provenance doesn't support.
The grouping of quotations into decisions is deliberately *not* done here — that grouping
is a reading, and readings belong where they can be argued with. Provenance is a record.

---

## 1. The vision statement

> *Pending. Assembled from the provenance below once the delta rounds settle. Writing it
> first would mean writing it from Claude's reading rather than from Ethan's words, which
> is the failure this structure exists to prevent.*

---

## 2. Provenance

Chronological. `P01` onward, stable — new quotations append, nothing is ever renumbered.
The vision statement and anything else cite these IDs rather than re-quoting.

Context appears where a quotation is incomplete without it — an answer needs its question.
It is drawn from what Claude actually asked at the time, and it stops there. Anything
beyond that is a reading, and readings live in
[`claude-scratchpad.md`](claude-scratchpad.md).

How the words arrived is recorded, because it changes how they should be read:

| | |
|---|---|
| `typed` | Ethan composed the words. He types precisely. |
| `voice` | Ethan spoke them. Voice-to-text, so transcription is imperfect; garbles are left as they stand. |
| `selected` | Ethan chose an option **Claude worded**. The decision is his; the words are not. |

Sources are the three conversations in [`transcripts/`](transcripts/).

---

## Conversation 1 — kickoff through the Action Phase

### P01 — The project · `typed` · 0001
> Hey Claude! Let's build a remake of Hearthstone: Battlegrounds. Would we be able to do a rust back end using this environment? I want to use such a project as an opportunity to build using Rust, but if you'd recommend otherwise for this concept, that would be fine.

### P02 — The three requirements, given as bullets in that same first message · `typed` · 0001
> - Seeded RNG
> - Data-Centric, Highly Configurable
>   Provided that the backend is data-oriented, attributes/parameters will be fairly accessible but, additionally, care should be taken to organize an intuitive config
> - Deep Backend, shallow frontend

### P03 — How to work · `typed` · 0001
> Throughout this project, please give me regular text updates on your progress and thoughts as you go.
>
> Also, you should delegate to Sonnet Subagents when useful, in the interest of token efficiency.

### P04 — Pace, after Claude wrote ~700 lines of schema before asking a question · `typed` · 0001
> You're getting ahead of me, haha! I want to flesh out the details collaboratively, especially the design.

### P05 — Roles · `typed` · 0001
> I'm primarily interested in serving the role of the designer & project manager, and I'm happy to grant you the role of the engineer. I'm primarily interested in the design aspect, but I'd like to be in the loop for the technical decisions - so maybe those could be multiple choice modals with the options and your recommendation.

### P06 — The vision, first statement · `typed` · 0001
> I want to make a minimalist core of the HS: Battlegrounds gameplay. I'll let you take the lead on probing me for the details of what/how I mean.

### P07 — What "minimalist" cuts, and the first list of rule changes · `typed` · 0001

*Q1 offered: (a) thin the content, keep the rules honest — "Full fidelity on mechanics —
Taunt, Divine Shield, Poisonous, Windfury, Deathrattles, tribes, tavern tiers, the
attack-alternation rule, cascading triggers — but 40 cards instead of 200"; (b) keep
content breadth, simplify the rules; (c) cut whole systems.*

> q1. A, but I'd like to modify some of the core rules. We could run through them in a modal. To cover a couple: instead of an 8 player FFA, I'd like to start with a simplified 1v1 model; instead of resolving combat back and forth in turns, I'd like each turn of attacks to resolve simultaneously (rock, paper, scissors is a real model); I'd like to add a constraint in the form of "conservation of values", like in currency, possibly in stat gains, possibly in minion generation, which sounds complex but I think would be straightforward, provided we have to calculate all of the numbers anyway; instead of being always-online I'd like the capacity for offline play; instead of being synchronous match-ups I'd like them to be asynchronous in a particular way; don't get too spooked if it sounds like a lot, I think on paper it will turn out to all be highly compatible and elegant; you mentioned a beauty and predictability in clean rules with unambiguous resolution, I totally agree and that's also part of the goal;

### P08 — An original Unit set · `typed` · 0001

*Q3 offered: (a) real Battlegrounds cards, (b) an original set using the same mechanics,
(c) real cards renamed later. Claude noted (a) would hand the project "a free correctness
oracle."*

> q3. B - the custom card set will be very limited, pretty much just a minimal set to demonstrate each effect. We'll do the equivalent of "box art" but with tokens.

### P09 — Milestones · `typed` · 0001

*Q4 asked what the first working moment is: (a) a combat resolves, (b) a full recruit turn,
(c) a whole game.*

> q4. Let's say v0.1 is A, v0.2 is B, v0.3 is C and the official 'prototype'

### P10 — Docs, and the transcript archive · `typed` · 0001
> On the project overall: in the spirit of the skills we're using, indeed it will be vital that you put together comprehensive docs and, most importantly, set up a workspace for yourself in which you can feel confident even an amnesiac instance could easily jump back into the work.
>
> In fact, something I've been wanting to try is maintaining a folder which contains full transcripts of our conversations - with such a folder, we could even nuke the repo, start an entirely fresh instance, and seed it with exactly the conversation that spawned the repo initially.

### P11 — Beats, not discrete actions · `typed` · 0001

*Q5 asked what mechanism simultaneity should use, offering: (a) positional pairing — "The
Boards line up facing each other. Position 1 fights position 1... No targeting decision
exists", which Claude recommended while flagging that "(a) kills Taunt"; (b) simultaneous
target selection; and (c) — "**One pair per Tick.** Closest to Battlegrounds — one Minion
from each side is selected and they trade — but simultaneously, so both always connect."*

> Q5: C but modified: I want to maintain the sense of a synchronous narrative playing out, one beat at a time, but I don't see it as necessary to separate the interaction into discrete actions.
> - Instead of proceeding in discrete 'actions' this app should proceed in 'beats'
> - Thinking through why it is I feel that way... It's because the 'autobattle' of an autobattler is a *simulation playing out* - the discrete 'actions' of autobattlers feels like a holdover from card games and TTRPG.

### P12 — Three resources, separate but correlated · `typed` · 0001

*Q6 asked what conservation of value conserves — (a) one currency, (b) several conserved
separately, (c) stats only — at what scope, and whether conservation is strict or bounded.*

> Q6: B but it's secretly sort of A: I adore resource management and I think the most interesting relationship for resources is to be technically separate, loosely correlated, but still correlated. So we'll start with: Eoconomy:Power:Minions
> - My favorite example is Hades - by and large the main split is in-run vs meta currency, which are almost uncorrelated, and then meta currency is multiple currencies which are technically liquid but very slow to exchange.
>   - Be mindful not to fixate on this example - I'm just sharing in the delight and providing an example of the potential depth.
> - At which scope?
>   - I: one scope, in-run (perhaps more, beyond prototype)
> - 6.3: Bounded: this is just intuition, but that feels right - tax big gains, limit losses;

### P13 — What asynchronous means · `typed` · 0001

*Q7 asked, with no recommendation: Claude said it would "rather hear it than build a
strawman."*

> Q7: asynchronous matches meaning unbounded prep phase, opponent is pre-selected from a pool randomly, action phase begins for a given player in response to ending their prep phase;

### P14 — Vocabulary is Ethan's · `typed` · 0001
> By the way, be sure to go through my messages and make note of the vocabulary I use

### P15 — Units, Pool, Action, Card · `typed` · 0001
> - 1v1 - you made a good point, we should, more or less, abandon this framing;
> - Instead of minions, let's go with units - I just decided I like units more as the general term
> - Pool - I think we'll probably want to leave this non-specified, given that there could be a pool of opponents, minions, effects, currency, etc;
> - Action - also probably non-specific, as "player action" makes as much sense as "unit action"
> - Card - I was going back and forth on this, but I'm settled on card; cards represent units, the physical representation of units will largely be cards, 1:1.6 cards. This is a nice improvement over Battlegrounds having 'tokens' randomly as play-pieces.
>   - (I'm willing to make the call that our visual direction is similar to HS: Battlegrounds, but moving toward Marvel Snap.)

### P16 — The vision, restated · `typed` · 0001
> It's probably worth restating the vision with how much I've deviated from Battlegrounds:
> - I'd like to make  a minimalist demake of Hearthstone: Battlegrounds with a number of key tweaks which make it into a distinct autobattler.
> - Any confusion or refooting that's happening is totally on me, by the way, I'm not like bothered by us finding misalignments because, after all, I'm changing my position like every message, hah!

### P17 — Left to right, same slot synchronous · `typed` · 0001

*Q8 asked whether every Unit acts every Beat, or whether Units have rates (speeds).
Claude recommended rates. Ethan rejected both.*

> Q8: Secret third option - similar to Battlegrounds but distinct, the action phase resolves from left to right and units in the same slot are synchronous.
> - This is to preserve the sense of watching a synchronous, cohesive narrative playing out

### P18 — Eight Slots, and the Party · `typed` · 0001
> Q8.1: slots? I'll make the call that a full 'party' of units should occupy 8 slots on the board
> - (I just like 8 &/or powers of 2)
> Q8.2: Party? The player is formulating a party consisting of units which occupy some number of 8 total slots.

### P19 — The default rule, and Battlegrounds' emergent economy · `typed` · 0001
> Q9: Let's plan on not having this figured out all the way at first (which moreso resembles option B from your options about currency), therefore we think of them as disconnected but correlated. Just think of them as roughly like battlegrounds values, and 1 is the anchoring value - the most basic, not exceptional unit of measure
> - For context, this is actually how I'd describe Battlegrounds in the abstract as well, it's not me inventing framing - in a sense, there is an emergent economy in a game like battlegrounds which exchanges currency for power, for units, for meta-progress; it's just that, in a game like battlegrounds, there wasn't as much intentionality behind the design and the balance was done 100% by hand
> - Also, I really am thinking of this as "gameplay vertical slice of battlegrounds" *but with* [these changes]; just to clarify, because perhaps it sounds like I'm describing a radically different game, but, in a sense, I'm taking refuge in the safety of "just however battlegrounds does it"

### P20 — Units as the third resource axis, and bounding · `typed` · 0001
> Q9.1: In "economy:power:units", 'units' is the discrete unit by which the player is constructing their party.
> Q9.2: It will be later on that we think about such dynamics, I'll say;
> Q9.3: I actually was just responding to the question - to bound how much damage can be dealt in a single action is to tax the big actor and cut the losses of the small actor.

### P21 — A stream of opposing Parties · `typed` · 0001

*Q10 asked whether there are still two Seats, or one Seat facing a stream. Claude
recommended the stream while flagging that it contradicted an ADR it had just written.*

> Q10: b - indeed, I think we should think of it as a stream of opposing parties.

### P22 — How abilities are expressed · `selected` · 0001

*The one decision in the corpus recorded in Claude's words rather than Ethan's. Claude
asked "How should a Unit's abilities be expressed in the codebase?", argued from a survey
Claude had itself commissioned and written, and recommended this option. Ethan selected
it. The decision is his; every word of it is Claude's.*

> Data + pipeline hook (Recommended)

### P23 — The sweep, deaths, and heroes · `typed` · 0001

*Q11 asked three things. 11.1: does the sweep repeat, or end after one pass? 11.2: "**When
do deaths apply?** At the end of the Beat that caused them, or at the end of a full
sweep?" — Claude recommending end of the Beat because "End-of-sweep means corpses keep
fighting, which is exactly the invisible bookkeeping we're removing." 11.3: what happens
where only one side has a Unit, Claude recommending the lone Unit strikes the Player.
Q12 asked whether heroes exist.*

> 11.1 loop, 11.2 end of beat, 11.3 yes, though we'll revisit later on, Q12 cut heroes;

### P24 — Taunt and Windfury are not casualties · `typed` · 0001

*Claude had declared both keywords meaningless under its reading of P17.*

> btw, if you revisit taunt and windfury, I think you'll find they still work out; turn resolution is left to right but it's valid for effects to have positional implications, and windfury means the action that a unit would do once in a beat, they do twice;

---

## Conversation 2 — corrections, and the delta register

### P25 — On the docs that existed then · `typed` · 0002
> Also, to be explicit, these docs were written by another instance of Claude and, for most of that session, the goal was quite loosely defined so they did a lot of hedging and stuff. After I made the goal more clear they had an easier time writing docs - the main thing to consider is that this project is "just like a gameplay vertical slice of HS: Battlegrounds except with [these tweaks]" these tweaks being the ~5 drastic gameplay modifications I made

### P26 — Four things not implied · `typed` · 0002
> I noticed a couple stray points that I've been neglecting which are getting in the way, time to get those settled. [...] I'm hoping you can track down based on that transcript these couple of points which I didn't mean to imply: I didn't mean to imply changing attack ordering, or imply changing attack targeting, or imply changing keyword behavior, or imply a completely deterministic board;

### P27 — Battlegrounds beside the demake, line by line · `typed` · 0002

*Answering "What should determine who fights whom within a Beat, now that fixed slot-vs-slot
pairing is off the table?"*

> Yeah, it's just an unfortunate disconnect on meaning and then me repeatedly missing signs of it - in the following I'll describe Battlegrounds, then I'll describe our modification: BG, attack order? Left to right, alternating sides. BG, targeting? Random, unless taunt is in play. BG, taunt? While a unit with taunt is in play in the opposing party, all attacks must target them. Demake, attack order? Left to right. Resolved simultaneously. Demake, targeting? Random, unless a unit with taunt is in play. Demake, taunt? If a unit with taunt is in play, attacks must target it. Demake, damage/death resolution? Simultaneous, attacker dies last. (This last one was an ambiguity I hadn't anticipated)

### P28 — No unit attacks the Player; a second attack re-draws · `typed` · 0002
> Further, very understandable misunderstandings to correct: Battlegrounds doesn't have units attacking the player (though Hearthstone does), a second attack is a second choosing-of-target.

### P29 — The Party is left-anchored · `typed` · 0002
> A new tweak:
> Battlegrounds doesn't have an elegant solution to unit placement, but I can imagine one.
> Instead of Battleground's ambiguity about positioning, this app will anchor the party on the left-most unit and occasionally compact toward them. Compaction is persistently applied in the prep phase, then applied scarcely in the action phase - at the start of a new round of beats OR before the first unit attacks. (Hearthstone & Battlegrounds have quirkiness to them because Hearthstone tries to maintain a "centered" party whereas Battlegrounds is anchored to the left by convention but still tethered by the engine in some cases)

### P30 — A Beat is a time-step · `typed` · 0002

*Claude had implemented per-Party Passes and flagged that as its own reading rather than
Ethan's rule.*

> Tweak: let's make canonical that a 'beat' is a time-step - so no per party passes. Instead, compaction is triggered before beat 1 - beat 0 is compaction, beat 1 is slot 1.

### P31 — Deltas, each traced to its message · `typed` · 0002
> Could you do a reassessment of the 'deltas' - the docs might start to get stilted if there's changes which aren't mentioned. It's good to tie the concept all together with multiple Deltas, I like that, but there's potential for misalignment, it would be good to track down every discrete delta mentioned, document it, and have each one link back to the message in which I introduced it.

---

## Conversation 3 — the transaction, and this rewrite

### P32 — Keywords are a straightforward adaptation · `typed` · 0003
> Take a look at where we left off, I believe we're concepting and then implementing keywords. If that's right, consider what would be highest impact / most instructive to ask about with keywords. In my mind, it's a straightforward adaptation, considering our goal

### P33 — What simultaneous means · `voice` · 0003

*Answering a round of keyword questions, each of which Ethan referred back to Battlegrounds.*

> The answer to each is to reference Battleground's functionality, and I'll elaborate on the simultaneity;
>
> when I say attacks resolve simultaneously, I mean that ordering doesn't grant any advantage. Every other consequence should follow relatively simply, and I want you to own the implementation while implementing according to my vision.
>
> Another thing worth addressing is that I never said that deaths resolve at the end of beats

### P34 — Steps, and putting adjacent ones in the same Beat · `voice` · 0003
> My modifications are principles.
>
> To say that these modifications are principles is exactly exactly right, um, because I'm not trying to massively change the underlying model that battlegrounds uses. For example, simultaneous works a little bit different than how you've just described it. Because, uh, when I say this, I mean, um, in contrast to battlegrounds, battlegrounds does, um, uh, like, a, uh, does steps instead of beats, and steps are like atomic, um, actions. And, uh, so that means a step can be, like, an attack or an effect or a trigger or an action or whatever. Um, what I'm suggesting is not a massive change. It's... instead of in battlegrounds when Opposing unit attacks resolves. Friendly unit attacks resolves. taking those steps which go right after one another and conceptually putting them in the same beat to remove the ordering consequence.

### P35 — A transaction, not a trade · `voice` · 0003

*Correcting Claude, which had collapsed two Units attacking each other into a single
symmetrical event.*

> in battlegrounds, there was effectively no such thing as a trade. One unit attacking the other, and then the other unit attacking them could play out asymmetrically because it's not a trade. It's a transaction. I think that makes sense. It's a, like, a a programmer bias of, like, one action, one outcome. But what we are aiming for here is, like, a more intuitive simulation type model. where it means something for two actions to be simultaneous. so I propose that we keep retaliation. We remove, um, any, like, pooling like the, uh, clash, and instead we treat it almost identically to battlegrounds. in which one side attacks, which implies retaliation. The other side attacks, which replies, I mean, implies retaliation. And those attacks resolve simultaneously because they happened simultaneously.

### P36 — On trusting Claude, and what the docs are for · `voice` · 0003
> I feel like I can trust Claude just as long as the policy is correct.

### P37 — Why Claude's own words come back with too much authority · `voice` · 0003
> not differentiating between what's my edict and your mental model is what creates issues because you read your words to be, um, meant in the way that a... that an instance of Claude would write them. I think that's where that misplace confidence comes from, and it's, like, not a terrible thing that you do.
