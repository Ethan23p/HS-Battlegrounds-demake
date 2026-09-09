# Vision

Two sections. The **vision statement** says what this project is. The **provenance
ledger** is the evidence for it: Ethan's own words, grouped by the decision they bear on.

The rule between them is mechanical, not a matter of judgment — **no claim in the vision
statement without an entry in the ledger behind it.** A claim that cannot cite one is a
claim Claude made up, and it does not belong here.

---

## 1. The vision statement

> *Pending. Assembled from the ledger below once the delta rounds settle — see
> [`claude-scratchpad.md`](claude-scratchpad.md) for where that stands. Writing it before
> the rounds would mean writing it from Claude's reading rather than from Ethan's words,
> which is the failure this whole structure exists to prevent.*

---

## 2. Provenance

### How to read this

Entries are grouped by **decision**, numbered `P1`…`Pn`, and chronological within each
group. Reading a group top to bottom is the history of that decision — including where it
changed, which is the part that kept getting lost when a document cited one quote and
dropped the other three.

IDs never change. A new quotation appends within its group; a new decision appends a
group. Nothing is renumbered, so a citation written today still resolves in a year.

Each entry carries how the words arrived, because it changes how they should be read:

| Tag | Meaning |
|---|---|
| `[typed]` | Ethan composed the words. He types precisely; read them closely. |
| `[voice]` | Ethan spoke them. Transcription is imperfect — garbles are flagged, never silently repaired. |
| `[selected]` | Ethan chose an option **Claude worded**. The decision is his; the words are not. |
| `[relayed]` | Text Ethan pasted but did not author. Carries no authority. Listed so it is not mistaken for his. |

**Context** appears only where a quotation is incomplete without it — an answer without
its question, a "yes" without what it answered. Never to explain, extend, or conclude.
Anything beyond that is Claude's reading and lives in the scratchpad.

**Sources.** Three conversations, five files. `0002` and `0003` are the canonical exports;
`0001` and `0005` are shorter snapshots of those same two sessions and are never cited.
`0006` is the session that produced this document.

Quotations are verbatim. Where a spoken turn contains a mistranscription, it is quoted as
it stands rather than repaired — a garbled quote is a better record than a smoothed one.

---

### P1 — What the project is

- **P1.1** [typed] — 0001 L18
  > Hey Claude! Let's build a remake of Hearthstone: Battlegrounds. Would we be able to do a rust back end using this environment? I want to use such a project as an opportunity to build using Rust, but if you'd recommend otherwise for this concept, that would be fine.

- **P1.2** [typed] — 0001 L18
  > - Seeded RNG
  > - Data-Centric, Highly Configurable
  >   Provided that the backend is data-oriented, attributes/parameters will be fairly accessible but, additionally, care should be taken to organize an intuitive config
  > - Deep Backend, shallow frontend

- **P1.3** [typed] — 0001 L245
  > I want to make a minimalist core of the HS: Battlegrounds gameplay. I'll let you take the lead on probing me for the details of what/how I mean.

- **P1.4** [typed] — 0001 L610
  > I'd like to make  a minimalist demake of Hearthstone: Battlegrounds with a number of key tweaks which make it into a distinct autobattler.

### P2 — The default rule: undeltered, it works however Battlegrounds works

- **P2.1** [typed] — 0001 L365
  > you mentioned a beauty and predictability in clean rules with unambiguous resolution, I totally agree and that's also part of the goal

- **P2.2** [typed] — 0001 L610
  > Also, I really am thinking of this as "gameplay vertical slice of battlegrounds" *but with* [these changes]; just to clarify, because perhaps it sounds like I'm describing a radically different game, but, in a sense, I'm taking refuge in the safety of "just however battlegrounds does it"

- **P2.3** [typed] — 0002 L61
  > the main thing to consider is that this project is "just like a gameplay vertical slice of HS: Battlegrounds except with [these tweaks]" these tweaks being the ~5 drastic gameplay modifications I made

- **P2.4** [typed] — 0002 L104
  > I'm hoping you can track down based on that transcript these couple of points which I didn't mean to imply: I didn't mean to imply changing attack ordering, or imply changing attack targeting, or imply changing keyword behavior, or imply a completely deterministic board;

### P3 — A Run against a stream, not a lobby

- **P3.1** [typed] — 0001 L365
  > instead of an 8 player FFA, I'd like to start with a simplified 1v1 model

- **P3.2** [typed] — 0001 L610
  > 1v1 - you made a good point, we should, more or less, abandon this framing;

- **P3.3** [typed] — 0001 L610
  > Q10: b - indeed, I think we should think of it as a stream of opposing parties.

### P4 — Asynchronous Rounds, and offline play

- **P4.1** [typed] — 0001 L365
  > instead of being synchronous match-ups I'd like them to be asynchronous in a particular way

- **P4.2** [typed] — 0001 L365
  > instead of being always-online I'd like the capacity for offline play

- **P4.3** [typed] — 0001 L506
  > Q7: asynchronous matches meaning unbounded prep phase, opponent is pre-selected from a pool randomly, action phase begins for a given player in response to ending their prep phase;

### P5 — Beats, and what "simultaneous" means

The longest group, and the one that drifted twice. Read it in order.

- **P5.1** [typed] — 0001 L365
  > instead of resolving combat back and forth in turns, I'd like each turn of attacks to resolve simultaneously (rock, paper, scissors is a real model)

- **P5.2** [typed] — 0001 L506
  > Q5: C but modified: I want to maintain the sense of a synchronous narrative playing out, one beat at a time, but I don't see it as necessary to separate the interaction into discrete actions.

- **P5.3** [typed] — 0001 L506
  > Instead of proceeding in discrete 'actions' this app should proceed in 'beats'

- **P5.4** [typed] — 0001 L506
  > Thinking through why it is I feel that way... It's because the 'autobattle' of an autobattler is a *simulation playing out* - the discrete 'actions' of autobattlers feels like a holdover from card games and TTRPG.

- **P5.5** [typed] — 0001 L610
  > Q8: Secret third option - similar to Battlegrounds but distinct, the action phase resolves from left to right and units in the same slot are synchronous.

- **P5.6** [typed] — 0002 L146
  > Demake, attack order? Left to right. Resolved simultaneously.

- **P5.7** [typed] — 0002 L263
  > Tweak: let's make canonical that a 'beat' is a time-step - so no per party passes. Instead, compaction is triggered before beat 1 - beat 0 is compaction, beat 1 is slot 1.

- **P5.8** [voice] — 0003 L103
  > when I say attacks resolve simultaneously, I mean that ordering doesn't grant any advantage.

- **P5.9** [voice] — 0003 L172
  > battlegrounds does, um, uh, like, a, uh, does steps instead of beats, and steps are like atomic, um, actions. And, uh, so that means a step can be, like, an attack or an effect or a trigger or an action or whatever.

- **P5.10** [voice] — 0003 L172
  > instead of in battlegrounds when Opposing unit attacks resolves. Friendly unit attacks resolves. taking those steps which go right after one another and conceptually putting them in the same beat to remove the ordering consequence.

### P6 — An attack is a transaction, and the attacker dies last

- **P6.1** [typed] — 0002 L146
  > Demake, damage/death resolution? Simultaneous, attacker dies last. (This last one was an ambiguity I hadn't anticipated)

- **P6.2** [voice] — 0003 L224
  > in battlegrounds, there was effectively no such thing as a trade. One unit attacking the other, and then the other unit attacking them could play out asymmetrically because it's not a trade. It's a transaction.

- **P6.3** [voice] — 0003 L224
  > so I propose that we keep retaliation. We remove, um, any, like, pooling like the, uh, clash, and instead we treat it almost identically to battlegrounds. in which one side attacks, which implies retaliation. The other side attacks, which replies, I mean, implies retaliation. And those attacks resolve simultaneously because they happened simultaneously.

### P7 — Targeting, Taunt, Windfury: not deltas

- **P7.1** [typed] — 0002 L146
  > BG, attack order? Left to right, alternating sides. BG, targeting? Random, unless taunt is in play. BG, taunt? While a unit with taunt is in play in the opposing party, all attacks must target them.

- **P7.2** [typed] — 0002 L146
  > Demake, targeting? Random, unless a unit with taunt is in play. Demake, taunt? If a unit with taunt is in play, attacks must target it.

- **P7.3** [typed] — 0001 L723
  > btw, if you revisit taunt and windfury, I think you'll find they still work out; turn resolution is left to right but it's valid for effects to have positional implications, and windfury means the action that a unit would do once in a beat, they do twice;

- **P7.4** [typed] — 0002 L211
  > Battlegrounds doesn't have units attacking the player (though Hearthstone does), a second attack is a second choosing-of-target.

### P8 — When deaths resolve

The drift case, kept as a group precisely because one entry alone caused it.

- **P8.1** [typed] — 0001 L723
  > 11.1 loop, 11.2 end of beat, 11.3 yes, though we'll revisit later on, Q12 cut heroes;

  Context (required — an answer is incomplete without its question). Claude asked:
  *"11.2 — When do deaths apply? At the end of the Beat that caused them, or at the end
  of a full sweep?"*, recommending *"End of the Beat. [...] End-of-sweep means corpses
  keep fighting, which is exactly the invisible bookkeeping we're removing."*

- **P8.2** [voice] — 0003 L103
  > Another thing worth addressing is that I never said that deaths resolve at the end of beats

### P9 — The Party is left-anchored

- **P9.1** [typed] — 0002 L211
  > Instead of Battleground's ambiguity about positioning, this app will anchor the party on the left-most unit and occasionally compact toward them. Compaction is persistently applied in the prep phase, then applied scarcely in the action phase - at the start of a new round of beats OR before the first unit attacks.

### P10 — Eight Slots, and the Party

- **P10.1** [typed] — 0001 L610
  > Q8.1: slots? I'll make the call that a full 'party' of units should occupy 8 slots on the board

- **P10.2** [typed] — 0001 L610
  > (I just like 8 &/or powers of 2)

- **P10.3** [typed] — 0001 L610
  > Q8.2: Party? The player is formulating a party consisting of units which occupy some number of 8 total slots.

### P11 — Three resources, bounded

- **P11.1** [typed] — 0001 L365
  > I'd like to add a constraint in the form of "conservation of values", like in currency, possibly in stat gains, possibly in minion generation, which sounds complex but I think would be straightforward, provided we have to calculate all of the numbers anyway

- **P11.2** [typed] — 0001 L506
  > Q6: B but it's secretly sort of A: I adore resource management and I think the most interesting relationship for resources is to be technically separate, loosely correlated, but still correlated.

- **P11.3** [typed] — 0001 L506
  > 6.3: Bounded: this is just intuition, but that feels right - tax big gains, limit losses;

- **P11.4** [typed] — 0001 L610
  > Just think of them as roughly like battlegrounds values, and 1 is the anchoring value - the most basic, not exceptional unit of measure

- **P11.5** [typed] — 0001 L610
  > in a sense, there is an emergent economy in a game like battlegrounds which exchanges currency for power, for units, for meta-progress; it's just that, in a game like battlegrounds, there wasn't as much intentionality behind the design and the balance was done 100% by hand

- **P11.6** [typed] — 0001 L610
  > Q9.1: In "economy:power:units", 'units' is the discrete unit by which the player is constructing their party.

- **P11.7** [typed] — 0001 L610
  > Q9.3: I actually was just responding to the question - to bound how much damage can be dealt in a single action is to tax the big actor and cut the losses of the small actor.

### P12 — Vocabulary

- **P12.1** [typed] — 0001 L506
  > By the way, be sure to go through my messages and make note of the vocabulary I use

- **P12.2** [typed] — 0001 L610
  > Instead of minions, let's go with units - I just decided I like units more as the general term

- **P12.3** [typed] — 0001 L610
  > Pool - I think we'll probably want to leave this non-specified, given that there could be a pool of opponents, minions, effects, currency, etc;

- **P12.4** [typed] — 0001 L610
  > Action - also probably non-specific, as "player action" makes as much sense as "unit action"

- **P12.5** [typed] — 0001 L610
  > Card - I was going back and forth on this, but I'm settled on card; cards represent units, the physical representation of units will largely be cards, 1:1.6 cards. This is a nice improvement over Battlegrounds having 'tokens' randomly as play-pieces.

### P13 — Content scope

- **P13.1** [typed] — 0001 L365
  > q3. B - the custom card set will be very limited, pretty much just a minimal set to demonstrate each effect. We'll do the equivalent of "box art" but with tokens.

- **P13.2** [typed] — 0001 L723
  > Q12 cut heroes;

### P14 — Milestones

- **P14.1** [typed] — 0001 L365
  > q4. Let's say v0.1 is A, v0.2 is B, v0.3 is C and the official 'prototype'

### P15 — Presentation

- **P15.1** [typed] — 0001 L610
  > (I'm willing to make the call that our visual direction is similar to HS: Battlegrounds, but moving toward Marvel Snap.)

### P16 — How abilities are expressed

The whole group is one `[selected]` entry, and that is the point: this decision has never
been stated in Ethan's own words. The wording below is Claude's — an option Claude wrote
and recommended, resting on an analysis Claude also wrote. Ethan chose it, so the decision
is genuinely his; the framing behind it has no independent basis in anything he said.

- **P16.1** [selected] — 0001 L696
  > Data + pipeline hook (Recommended)

  Context (required — a selected label is meaningless without its question). Claude asked:
  *"How should a Unit's abilities be expressed in the codebase?"*

### P17 — The working agreement

- **P17.1** [typed] — 0001 L45
  > Throughout this project, please give me regular text updates on your progress and thoughts as you go.

- **P17.2** [typed] — 0001 L45
  > Also, you should delegate to Sonnet Subagents when useful, in the interest of token efficiency.

- **P17.3** [typed] — 0001 L245
  > I'm primarily interested in serving the role of the designer & project manager, and I'm happy to grant you the role of the engineer. I'm primarily interested in the design aspect, but I'd like to be in the loop for the technical decisions - so maybe those could be multiple choice modals with the options and your recommendation.

- **P17.4** [typed] — 0001 L365
  > it will be vital that you put together comprehensive docs and, most importantly, set up a workspace for yourself in which you can feel confident even an amnesiac instance could easily jump back into the work.

- **P17.5** [typed] — 0001 L365
  > something I've been wanting to try is maintaining a folder which contains full transcripts of our conversations - with such a folder, we could even nuke the repo, start an entirely fresh instance, and seed it with exactly the conversation that spawned the repo initially.

- **P17.6** [typed] — 0002 L61
  > these docs were written by another instance of Claude and, for most of that session, the goal was quite loosely defined so they did a lot of hedging and stuff.

- **P17.7** [typed] — 0002 L307
  > It's good to tie the concept all together with multiple Deltas, I like that, but there's potential for misalignment, it would be good to track down every discrete delta mentioned, document it, and have each one link back to the message in which I introduced it.

---

## Excluded from the ledger

Text that appears in Ethan's turns but is not his, listed so a later instance does not
find it and wonder why it is missing.

- **0002 L838** `[relayed]` — a message authored by a different Claude instance, working on
  branch `claude/reduce-docs-size-1ggc5c`, which Ethan pasted in to relay. Technical
  instructions about the transcript exporter. Not his words; carries no authority.
- **0003 L461** `[relayed]` — a report from a sibling Claude session, beginning "Pushed as
  `ed71a3d`", quoted by Ethan. The one line around it that *is* his — *"I also made a
  "main" branch, so at the next opportunity you can create a pull request for that."* — is
  his, and is not a design decision.
- Turns consisting only of the exporter's `*(user interrupted Claude's previous turn)*`
  annotation, which contain no words of Ethan's at all.
- `<local-command-stdout>` blocks, which are harness output.
