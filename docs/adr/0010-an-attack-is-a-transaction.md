---
status: accepted
---

# An attack is a transaction, and simultaneity is a principle rather than a mechanism

Two rules the engine was enforcing had never been decided. One was a mechanic that
quietly went missing; the other was a mechanic that quietly appeared. Both are corrected
here, and both were found by asking what the simultaneity delta actually *says*.

**Ethan's own statement of the delta, which is what this ADR is built on:**

> when I say attacks resolve simultaneously, I mean that ordering doesn't grant any
> advantage.
>
> — [transcript 0006](../transcripts/0006-the-exchange-and-the-instance.md)

That is a principle, not a mechanism, and it cuts far more narrowly than the mechanism
[ADR 0008](0008-targeting-is-random-simultaneity-is-the-only-delta.md) wrote down.
Battlegrounds' outcomes are mostly *already* independent of ordering, because its attacks
are mutual. The delta's job is to remove the one place ordering does decide something —
who swings first — and to leave everything else exactly where Battlegrounds left it.
Everything below is Claude's engineering of that principle, not further instruction.

## What went missing: the answering blow

**In Battlegrounds an attack damages both Units.** The attacker deals its attack to its
target, and the target deals its attack back, in the same instant. The engine only ever
damaged the target. Nothing decided that; it appears to have fallen out when ADR 0008
removed slot-vs-slot pairing, and no test caught it, because every scenario in the suite
was effectively 1v1 — where both Units are their side's Slot-1 attacker, so an answering
blow is indistinguishable from the other side's attack.

The cost was not a rounding error. Every keyword is priced by it:

| | With the exchange | Without it, as built |
|---|---|---|
| **Poisonous** | Mostly *defensive* — attack into it and die | Offensive only, and only on its own Beat |
| **Divine Shield** | Absorbs an answering blow, so attacking is free | Only ever blocks an incoming attack |
| **Taunt** | Punishes attackers; a fat-attack Taunt is a threat | A damage sponge with no downside for the attacker |
| **Windfury** | Doubles damage *and* doubles exposure | Strictly double damage, no added risk |

It is restored: a Unit's health, Taunt and Poisonous now mean something while it is not
the one swinging. Answering is **not** attacking — it draws no target of its own, it does
not make the answering Unit an attacker for "attacker dies last," and it will not fire an
`OnAttack` Trigger when Triggers exist.

## What appeared: deaths at the end of the Beat

The engine held corpses on the Board until a Beat finished, so a Unit killed by a
Windfury holder's first swing was still standing for the second. **Battlegrounds resolves
a death immediately after the attack that caused it.**

The record is worth being precise about, because this is the second time a rule has
drifted this way. Ethan *did* answer a question with "end of beat" — but the question was
"**at the end of the Beat that caused them, or at the end of a full sweep?**", asked when
a Beat was still one full exchange of the whole Board, and his stated reason was against
corpses lingering at all:

> **11.2 — When do deaths apply?** At the end of the Beat that caused them, or at the end
> of a full sweep? ➡️ **End of the Beat.** [...] End-of-sweep means corpses keep fighting,
> which is exactly the invisible bookkeeping we're removing.
>
> — Claude's question, [transcript 0002](../transcripts/0002-vocabulary-and-action-phase.md); answered *"11.2 end of beat"*

He chose the option that kept corpses off the Board. When ADR 0008 redefined a Beat as
"Slot *n* acts, twice if it has Windfury," the phrase he had agreed to silently came to
mean the thing he had rejected. His correction — *"I never said that deaths resolve at
the end of beats"* ([0006](../transcripts/0006-the-exchange-and-the-instance.md)) — is
right about the decision, whatever the words looked like.

## The mechanism: steps in a Beat

Battlegrounds resolves combat as a sequence of **steps** — atomic units of resolution. A
step is an attack, an effect, a trigger, a summon. Combat is: run the next step, fully,
then the next. Ethan's delta does not touch what a step is or how one resolves:

> instead of in battlegrounds when Opposing unit attacks resolves. Friendly unit attacks
> resolves. taking those steps which go right after one another and conceptually putting
> them in the same beat to remove the ordering consequence.
>
> — [transcript 0006](../transcripts/0006-the-exchange-and-the-instance.md)

A Beat is a **container for steps Battlegrounds would have run consecutively**, and the
point of the container is that nothing inside it can pre-empt anything else inside it.
That framing reaches past attacks: "two Deathrattles fire in the same Beat" is the
identical question, which is why it is worth stating at this level rather than as a rule
about damage.

For the Action Phase, one Beat holds one attack step from each side:

1. Both sides' Slot-*n* Unit draws its target, against the Board as the instance found it.
   Drawing before either attack resolves is the whole of what stops one from pre-empting
   the other.
2. Each attack resolves, fully, as Battlegrounds resolves an attack — including the
   answering blow.
3. The instance's dead are removed, a Unit that attacked after every Unit that didn't.

An *instance* is one pass through those three; a Beat is one instance, or two where
Windfury is involved. Deaths therefore resolve after the attacks that caused them and
before the next instance, so a Unit killed in instance 0 has no instance 1 to act in.
"End of the Beat" stops being a rule the engine has at all.

## An attack is a transaction, not a trade

This ADR originally recorded the opposite, and the correction is the more instructive
half of it. Two Units that choose each other in the same Beat were collapsed into a
single symmetrical "clash" that resolved once. The argument was that Battlegrounds leaves
a 3/4 trading with a 3/3 at 1 health whichever side swings first, so the outcome does not
turn on ordering and the delta should not change it.

The argument is wrong, and Ethan named why:

> in battlegrounds, there was effectively no such thing as a trade. One unit attacking the
> other, and then the other unit attacking them could play out asymmetrically because it's
> not a trade. It's a transaction.
>
> — [transcript 0006](../transcripts/0006-the-exchange-and-the-instance.md)

An attack has a **direction**: this Unit swings at that one, and the one struck answers.
Two attacks are two transactions, each with its own attacker. The reason the 3/4 wins in
Battlegrounds is that the 3/3 died before its own attack ran — which is *pre-emption*. It
happens to produce the same number whichever side goes first, so it looks
order-independent, but the mechanism is ordering, and removing pre-emption is precisely
the delta. Protecting that outcome reinstated the thing the delta deletes, and dressed it
up as fidelity.

Ethan diagnosed the collapse as a programmer's reflex — *"a programmer bias of, like, one
action, one outcome"* — against a model that is meant to be a simulation, *"where it means
something for two actions to be simultaneous."* Worth recording as a failure mode and not
just a wrong answer: the symmetric vocabulary came first (*trade*, *meeting*, *clash*),
and the symmetric mechanic followed from it. Naming a thing wrongly is how it gets built
wrongly.

So both attacks resolve. A 3/4 and a 3/3 destroy each other, and surviving a trade takes
more health than *twice* their attack — a real consequence, and the intended one.

## What this cost, and what it bought back

Collapsing the pair was not the only invention it forced. Pooling a Beat's damage to
resolve it as one event raised a question nothing else asks: what a Divine Shield absorbs
when two blows land at the same instant. This ADR answered it — the shield absorbs the
instant — and that answer is now gone with the pooling that needed it. One blow, one
shield, exactly as in Battlegrounds. **The invention only ever existed to serve the
implementation that invented it**, which is a useful smell: a rule Battlegrounds has no
opinion on is more often a sign of an over-built mechanism than of a genuine gap.

## What this does not change

- **Targeting stays random, and Taunt still constrains it** (ADR 0008).
- **"Attacker dies last" stands**, and finally has something to bite on: with the exchange
  restored, a mutual kill is what a trade usually *is*, not a coincidence of two Units
  drawing each other. It is Ethan's own call — *"Simultaneous, attacker dies last"*
  ([0003](../transcripts/0003-action-phase-corrections.md)) — and it is symmetric between
  the sides, so it grants neither one an advantage.
- **The clock is untouched.** Beat 0 closes ranks, Beat *n* resolves Slot *n*
  (ADR 0009). A Unit acts when the clock reaches its Slot, whatever put it there —
  summoned, returned by Reborn, or standing there since the Prep Phase.
- **The keyword set stays at five.** Battlegrounds' Frenzy, Avenge and Magnetic are
  Triggers, which the vocabulary already has; Cleave is ability text, expressible through
  `Selector::Adjacent`; Venomous is a real candidate that no content yet needs.

## Supersedes

[ADR 0008](0008-targeting-is-random-simultaneity-is-the-only-delta.md) on two points:
its "removal is deferred to the end of the Beat," and its comparison table's silence on
what an attack damages. Everything else in it — random targeting, Taunt, Windfury,
never attacking a Player, attacker dies last — stands unchanged.
