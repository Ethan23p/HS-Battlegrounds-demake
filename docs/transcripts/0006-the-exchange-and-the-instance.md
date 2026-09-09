---
session_id: 5ab60e7d-39c4-58d3-b35a-8ea9bd785988
export_date: 2026-09-09T14:44:49Z
source_path: /root/.claude/projects/-home-user-HS-Battlegrounds-demake/5ab60e7d-39c4-58d3-b35a-8ea9bd785988.jsonl
include_tools: false
message_counts:
  lines_total: 268
  user_turns: 2
  assistant_turns: 94
  user_lines_filtered_as_noise: 46
  tool_calls_omitted: 47
  tool_results_omitted: 46
  answers_recovered: 0
  queued_messages_recovered: 0
  thinking_blocks_omitted: 29
  parse_errors: 0
  unknown_block_shapes: 0
---

# Transcript: 5ab60e7d-39c4-58d3-b35a-8ea9bd785988

## User

Hey Claude! Take a look at where we left off, I believe we're concepting and then implementing keywords. If that's right, consider what would be highest impact / most instructive to ask about with keywords. In my mind, it's a straightforward adaptation, considering our goal

## Claude

I'll get oriented first — read the docs in the prescribed order, then look at the engine's current shape.

Let me settle the one open keyword item that's a *fact*, not a decision, before I write the round.

Oriented. Here's where things actually stand, then the round.

## Where we left off

Your memory is half right, and the half that's off is the interesting part.

The five keywords — Taunt, Divine Shield, Poisonous, Windfury, Reborn — are **already built and tested** in `crates/bg-sim/src/action_phase.rs`. `docs/scratchpad/state.md` lists them as live, and the suite is green. So "concept then implement keywords" isn't ahead of us as a build task; what's ahead is **Abilities** (Triggers, Effects, Selectors).

But going looking for the keyword frontier turned up something bigger than a keyword question, so I'll lead with it.

**One fact I settled rather than asking you:** open item 1 in the scratchpad was Reborn's return stats. Battlegrounds returns the minion with its current attack and 1 health, keeping other keywords, losing Reborn itself — which is exactly what `remove_and_log` already does. Considering it closed unless you disagree. ([wiki.gg](https://hearthstone.wiki.gg/wiki/Battlegrounds/Reborn), [fandom](https://hearthstone.fandom.com/wiki/Battlegrounds/Reborn))

---

❓ **Q1** — **Does a defender strike back?** In Battlegrounds an attack is an *exchange*: the attacker deals its attack to the defender, and the defender deals its attack back to the attacker, at the same instant. Our engine doesn't do this. `strike()` only ever mutates the target; there is no code path that damages an attacker. I ran a probe to be sure:

```
-- pass 1, beat 1 --
  player slot 1 strikes opposing slot 2 for 10
  opposing slot 1 strikes player slot 1 for 1
  opposing y in slot 2 dies
```

A 10/10 killed a 1/1 and took **nothing** back from it. In Battlegrounds it takes 1. Our 10/10 only ever gets hurt by whoever happens to occupy the enemy's Slot 1.

This was never a decision — no delta claims it, ADR 0008's comparison table doesn't mention it, and "the attacker dies last" (delta 5) is a rule that barely has anything to bite on without it: under the current model a mutual kill requires two Units to coincidentally draw each other, rather than being what every trade *is*. It looks like retaliation quietly fell out when slot-vs-slot pairing was removed, and nobody noticed because every test scenario is 1v1, where "both sides' Slot 1 attack each other" is indistinguishable from an exchange.

It re-prices every keyword, which is why it's a keyword question:

| | Battlegrounds (exchange) | Ours today (one-way) |
|---|---|---|
| **Poisonous** | Mostly *defensive* — attack into poison and die | Purely offensive, and only on its own Beat |
| **Divine Shield** | Eats a retaliation, so attacking is free | Only ever blocks an incoming attack |
| **Taunt** | Punishes attackers, so a fat-attack Taunt is a threat | A pure damage sponge with no downside for the attacker |
| **Windfury** | Doubles damage *and* doubles exposure | Strictly double damage, no added risk |

➡️ **Restore the exchange.** Delta 4 is about *sequencing between the two sides*, not about whether an attack is mutual; the default rule should be doing its job here and isn't. Concretely: when Slot *n* attacks, its target deals its own attack back simultaneously, and both deaths land at end of Beat with attacker-dies-last still deciding the order — which is finally the rule it was written to be. I'd write it up as a correction to ADR 0008 rather than a new delta, since it's restoring Battlegrounds rather than changing it.

---

❓ **Q2** — **Windfury's second swing, when the first one killed you.** Right now a Windfury Unit fatally wounded during instance 0 does *not* get instance 1: eligibility is re-checked against post-instance-0 state. That matches Battlegrounds (deaths process between attacks) but sits oddly beside our own "a dying Unit still lands its blow," which holds *within* an instance. So a Unit dying in the same instant it swings gets its blow; a Unit dying in instance 0 loses swing 2. Adding retaliation (Q1) makes this fire constantly instead of never.

Related, and flagged in the scratchpad already: a Windfury Unit whose second swing finds the enemy Party empty currently hits nothing rather than the Player — that one I'm confident is right and just needs your yes.

➡️ **Keep both as they are.** "Deaths apply at end of Beat, but eligibility to act is checked per instance" is Battlegrounds' behaviour and stays unambiguous under simultaneity. I'd just say it out loud in `CONTEXT.md` so it stops being an accident.

---

❓ **Q3** — **Is the keyword set closed at five?** Battlegrounds' own list is longer, but most of the remainder aren't keywords in our sense — `CONTEXT.md` defines a Keyword as a persistent property the Action Phase consults *directly*, and Frenzy, Avenge and Magnetic are all Triggers, which our vocabulary already has (`OnSurviveDamage`, `AfterFriendlyDeath`). That leaves two genuine candidates:

- **Venomous** — Poisonous that spends itself on one kill. Battlegrounds has both, and the pair is a real design axis (a renewable threat vs. a one-shot one).
- **Cleave** — splash to the target's neighbours. Battlegrounds treats this as ability text, not a keyword, and our `Selector::Adjacent` already exists to express it.

➡️ **Five for v0.1; Cleave is an Ability, not a keyword; Venomous deferred.** Cleave through `Adjacent` is exactly the "one Unit per Effect" content plan, and it exercises the Selector machinery we're about to build. Venomous only earns a keyword slot once there's enough content for the contrast to be felt — it costs an engine match arm for a distinction nothing currently uses.

---

❓ **Q4** — **When does a Unit that arrives mid-Pass act?** Reborn puts a Unit back into its own Slot during a Beat. If that Slot's Beat has already passed, it waits for the next Pass; if the clock hasn't reached it, it acts this Pass. That's what the code does today, by falling out of ADR 0009 rather than by anyone deciding it — and it's the *same* question that's about to hit summons the moment Deathrattles land (the scratchpad records it as an unverified expectation).

➡️ **Ratify one rule covering both: a Unit acts when the clock reaches its Slot, whatever put it there.** No special case for reborn Units, no special case for summons, no "arrived this Pass" bookkeeping. It's the cleanest reading of the Beat-as-time-step delta, and settling it now means Deathrattle summons need no new rule at all.

---

❓ **Q5** — **Do Action-Phase changes survive the fight?** Damage, popped shields, spent Reborn, and (once Effects exist) granted keywords all mutate Units in place, and `Resolution::final_board` hands back the mutated Board. Battlegrounds resets: your Party goes into the next Prep Phase as it was. Nothing enforces that yet because nothing outside the Action Phase exists.

➡️ **The Action Phase gets a copy; the Party of record is untouched.** Cheapest place to make it structural rather than a discipline the Prep Phase has to remember, and it keeps `final_board` honest as a read-only record of how the fight ended.

---

Nothing committed — this is a design round, so the branch is still clean. Q1 is the one I'd most want your answer on before touching code; Q3 and Q4 shape the Abilities work directly, so they're worth settling in the same breath.

## User

The answer to each is to reference Battleground's functionality, and I'll elaborate on the simultaneity;

when I say attacks resolve simultaneously, I mean that ordering doesn't grant any advantage. Every other consequence should follow relatively simply, and I want you to own the implementation while implementing according to my vision. 

Another thing worth addressing is that I never said that deaths resolve at the end of beats

## Claude

Understood on all three. Let me nail the last one down first — you're right, and I want to see exactly where that came from before I touch code.

You're right, and the record is more specific than either of us said. Let me put the exact quote up before I build on it.

Record is clear now. Let me read the rest of the engine, then build.

The record backs you up precisely, and the drift is worse than "I invented it" — I'll come back to that. First, the engine.

Four failures — and one of them is telling me something. Let me look at what the exchange actually did to a 1v1:

That test is the check on my own work. A 3/4 against a 3/3 in Battlegrounds leaves the 3/4 at 1 health *whichever side swings first* — the outcome doesn't turn on ordering, so my delta must not touch it. My first cut broke it by letting a mutually-targeting pair exchange twice. That's the fix:

All 25 pass untouched — which also means *nothing in the suite was covering retaliation*, because in a 1v1 both Units are attackers and the clash looks identical to the old one-way model. That's exactly why this hid. New tests:

My scenarios, not the engine — I wrote the assertions as if the fight stopped after one Beat. Let me read what actually happens:

Engine's right; my expectations were sloppy. Fixing the three scenarios:

Now the correction you flagged — deaths, and whether corpses fight on:

60 tests green, clippy clean. Now the documentation — starting with the ADR conventions:

Now the ADR. This is exactly the kind of correction the ADR directory exists for:

Now the pointers from ADR 0008, the vocabulary, the vision, and the scratchpad:

Now the scratchpad — the open questions this closes:

The ADR cites this conversation, so the transcript needs to exist:
