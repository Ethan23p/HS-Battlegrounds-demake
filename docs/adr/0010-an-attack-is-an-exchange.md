---
status: accepted
---

# An attack is an exchange, and simultaneity is a principle rather than a mechanism

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

## What went missing: the exchange

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

## The mechanism: the instance

The indivisible step is the **instance**, not the Beat. A Beat is one instance, or two
where Windfury is involved. Within one instance:

1. Both sides' Slot-*n* Unit declares an attack against the Board as the instance found
   it, drawing its own target. Neither side's draw can see the other's blow.
2. Every declared attack becomes a **clash** between two Units.
3. Each Unit in a clash deals its attack to the other, and all of an instance's damage
   lands at once.
4. The instance's dead are removed, a Unit that attacked after every Unit that didn't.

Deaths therefore resolve after the attack that caused them, exactly as in Battlegrounds,
and a Unit killed in instance 0 has no instance 1 to act in. "End of the Beat" stops being
a rule the engine has at all.

## Two tie-breaks simultaneity forces, which Battlegrounds cannot settle

The default rule says: undeltered, it works however Battlegrounds works. These are the
two places it has nothing to say, because the situation cannot arise there. In both, the
rule is chosen so that no ordering could change the outcome — which is the delta itself,
applied to its own consequences.

**A pair meets once per instance, however many of them swung.** When two Units choose
each other, Battlegrounds would resolve two separate attacks — but only ever gets to when
the first one failed to kill the second attacker. Its answer for a 3/4 trading with a 3/3
is the same whichever side swings first: the 3/4 lives at 1. That outcome does not turn
on ordering, so the delta has no business changing it. Counting the pair twice would deal
6 damage where Battlegrounds deals 3 — a rule change smuggled in as an implementation
detail. Collapsing the pair keeps every outcome Battlegrounds was already unambiguous
about and changes only the ones that turned on who swung first.

**A Divine Shield absorbs the instant, not one blow of it.** A Unit can now be struck by
one enemy and answer another in the same moment; nothing in Battlegrounds damages a Unit
twice at the same instant, so it never has to rule on which blow a shield eats. Absorbing
one would mean absorbing whichever an implementation happened to apply first, and the
blows may differ in size — an ordering advantage, in the one Action Phase built to have
none.

The second is the more debatable of the two, and it is the more generous reading: a
shield facing two blows at once now saves its holder from both. The alternative that
preserves order-independence — the shield absorbs the *largest* blow and the rest land —
is defensible, and cheap to switch to. Nothing else in the engine depends on which is
chosen.

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
