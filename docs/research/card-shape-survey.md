# Battlegrounds Card Shape Survey

**Purpose:** Settle, with facts rather than opinions, how much of the real Battlegrounds
minion pool can be expressed as pure parameterized data (a fixed vocabulary of "effect"
primitives combined declaratively), and precisely what resists.

**Method:** 44 minions sampled across all six tavern tiers and ten tribes plus neutrals,
favoring cards that are/were staples that define how the game plays over one-offs. Card
text was verified via web search against Blizzard's card library, hearthstone.wiki.gg,
HearthPwn, and Hearthstone Top Decks snapshots as they came back in search results
(direct page fetches were blocked by this environment's network egress policy, so
verification is via search-result snippets, not direct page reads). Cards are drawn from
across BG's history — several are currently rotated out of the live pool — because the
question is about the *shape* of the ability space the game has committed to, not this
week's exact roster. Cards I could not verify via search this session are marked
**UNVERIFIED** and are flagged as drawn from memory; treat their exact wording with more
skepticism than the verified entries. No code in the repository was read, written, or
run for this task.

---

## 1. Card-by-card catalogue

Each row: Tribe / Tier (approximate for a few cards — search snippets sometimes
disagreed on tier, text was prioritized), verified text, and the template it was
classified into.

| # | Card | Tribe | Tier | Status | Text | Template |
|---|------|-------|------|--------|------|----------|
| 1 | Alleycat | Beast | 1 | Verified | Battlecry: Summon a 1/1 Cat. | TOKEN-SUMMON-ON-TRIGGER |
| 2 | Murloc Tidehunter | Murloc | 1 | UNVERIFIED | Battlecry: Summon a 1/1 Murloc Scout. | TOKEN-SUMMON-ON-TRIGGER |
| 3 | Rockpool Hunter | Murloc | 1 | UNVERIFIED | Battlecry: Give a friendly Murloc +1/+1. | BATTLECRY-TRIBE-BUFF |
| 4 | Wrath Weaver | Demon | 1 | Verified | After you play a Demon, deal 1 damage to your hero and gain +2/+1, twice. | PLAY-TRIGGER-TRIBE-SELF-EFFECT |
| 5 | Imprisoner | Demon | 1 | Verified | Taunt. Deathrattle: Summon a 2/2 Imp. | TOKEN-SUMMON-ON-TRIGGER |
| 6 | Red Whelp | Dragon | 1 | Verified | Start of Combat: Deal 1 damage for each friendly Dragon to a random enemy minion, twice. | START-OF-COMBAT-TRIBE-SCALED-DAMAGE |
| 7 | Micro Mummy | Mech/Undead | 1 | Verified | Reborn. At the end of your turn, give another random friendly minion +1 Attack. | END-OF-TURN-BUFF |
| 8 | Rat Pack | Beast | 2 | Verified | Deathrattle: Summon a number of 1/1 Rats equal to this minion's Attack. | TOKEN-SUMMON-ON-TRIGGER (self-attack-scaled N) |
| 9 | Scavenging Hyena | Beast | 2 | Verified | Whenever a friendly Beast dies, gain +2/+1. | ON-DEATH-OF-TRIBE-SELF-BUFF |
| 10 | Imp Gang Boss | Demon | 2 | Verified | Whenever this minion takes damage, summon a 2/2 Imp. | TOKEN-SUMMON-ON-TRIGGER |
| 11 | Sellemental | Elemental | 2 | Verified | When you sell this, get two 3/3 Elementals. | ON-SELL-REWARD |
| 12 | Tough Tusk | Quilboar | 2 | Verified | After a Blood Gem is played on this, gain Divine Shield for the next combat. | **RESISTING** — resource-subsystem trigger |
| 13 | Toxfin | Murloc | 1–5 (snippets disagreed) | Verified | Battlecry: Give a friendly Murloc Poisonous. | BATTLECRY-TRIBE-BUFF (keyword variant) |
| 14 | Deflect-o-Bot | Mech | 3 | Verified | Divine Shield. Whenever you summon a Mech during combat, gain +4 Attack and Divine Shield. | TRIBE-SUMMON-TRIGGER-BUFF (target = self) |
| 15 | Security Rover | Mech | ~5 | Verified | Whenever this minion takes damage, summon a 2/3 Mech with Taunt. | TOKEN-SUMMON-ON-TRIGGER |
| 16 | Iron Sensei | Mech | 3 | Verified | At the end of your turn, give another friendly Mech +4/+4 (permanently). | END-OF-TURN-BUFF |
| 17 | King Bagurgle | Murloc | 4 | Verified | Battlecry: Give all other Murlocs in your hand and board +2/+3. | BATTLECRY-TRIBE-BUFF (broad-scope variant) |
| 18 | Southsea Strongarm | Pirate | 4 | Verified | Battlecry: Give a friendly Pirate +1/+1. Repeat for each Pirate you bought this turn. | **RESISTING** — turn-scoped economy counter as magnitude |
| 19 | Pack Leader | Beast | 3 | Verified | Whenever you summon a Beast, give it +4 Attack. | TRIBE-SUMMON-TRIGGER-BUFF (target = summoned minion) |
| 20 | Mama Bear | Beast | 6 | Verified | Whenever you summon a Beast, give it +12/+12. | TRIBE-SUMMON-TRIGGER-BUFF (target = summoned minion) |
| 21 | Goldrinn, the Great Wolf | Beast | 6 | Verified | Deathrattle: For the rest of this combat, your Beasts have +12/+12. | **RESISTING / borderline** — combat-scoped (not permanent, not a recomputed aura) buff duration |
| 22 | Waxrider Togwaggle | Dragon | 4 | Verified | Whenever a friendly Dragon kills an enemy, gain +2/+2. | **RESISTING** — needs kill attribution mid-combat |
| 23 | Wildfire Elemental | Elemental | 3 | Verified | After this attacks and kills a minion, deal excess damage to an adjacent enemy. | **RESISTING** — needs overkill value + adjacency |
| 24 | Herald of Flame | Elemental | 4 | Verified | Overkill: Deal 3 damage to the left-most enemy minion. | **RESISTING** — needs an Overkill hook + positional target |
| 25 | Cave Hydra | Beast | 4 | Verified | Also damages the minions next to whomever this attacks. | **RESISTING-ish** — cleave requires live board adjacency |
| 26 | Maexxna | Beast | 6 | Verified | Poisonous. (No other text.) | STATIC-KEYWORD-ONLY |
| 27 | Baron Rivendare | Undead | 6 | Verified | Your minions' Deathrattles trigger twice. | GLOBAL-RULE-MODIFIER |
| 28 | Khadgar | (non-tribal) | 6 | Verified | Your cards that summon minions summon twice as many. | GLOBAL-RULE-MODIFIER |
| 29 | Brann Bronzebeard | (non-tribal) | 5 | Verified | Your Battlecries trigger twice. | GLOBAL-RULE-MODIFIER |
| 30 | Murozond | Dragon | 5 | Verified | Battlecry: Get a plain copy of a minion from your last opponent's warband. | **RESISTING** — reads another player's board from a prior round |
| 31 | Kangor's Apprentice | Mech | 5 | Verified | Deathrattle: Summon your first 2 Mechs that died this combat. | **RESISTING** — ordered, filtered per-combat death log |
| 32 | Selfless Hero | (non-tribal) | 1 | Verified | Deathrattle: Give a random friendly minion Divine Shield. | DEATHRATTLE-RANDOM-TARGET-KEYWORD |
| 33 | Ghastcoiler | (non-tribal) | 5 | Verified | Deathrattle: Summon 2 random Deathrattle minions. | DEATHRATTLE-SUMMON-RANDOM-FILTERED |
| 34 | Slitherspear, Lord of Gains | Naga | 6 | Verified | At the end of your turn, give your other Naga +1/+1, improved by each different spell you've cast this turn. | **RESISTING** — turn-scoped "distinct spell types cast" counter |
| 35 | Replicating Menace | Mech | 4 | Verified | Magnetic. Deathrattle: Summon three 1/1 Microbots. | TOKEN-SUMMON-ON-TRIGGER, **plus RESISTING** for the Magnetic play-rule itself |
| 36 | Crowd Favorite | (non-tribal) | 4 | Verified | Whenever you play a card with Battlecry, gain +2/+2. | PLAY-TRIGGER-BY-KEYWORD-SELF-BUFF |
| 37 | Houndmaster | Beast | 2 | UNVERIFIED | Battlecry: Give a friendly Beast +2/+2 and Taunt. | BATTLECRY-TRIBE-BUFF |
| 38 | Metaltooth Leaper | Mech | 2 | UNVERIFIED | Battlecry: Give your other Mechs +2 Attack. | BATTLECRY-TRIBE-BUFF (board-wide variant) |
| 39 | Savannah Highmane | Beast | 4 | UNVERIFIED | Deathrattle: Summon two 3/3 Hyenas. | TOKEN-SUMMON-ON-TRIGGER |
| 40 | Cobalt Guardian | Mech | 3 | UNVERIFIED | Whenever you summon a Mech, gain Divine Shield. | TRIBE-SUMMON-TRIGGER-BUFF (target = self, keyword-only) |
| 41 | Coldlight Seer | Murloc | 2 | UNVERIFIED | Battlecry: Give your other Murlocs +2 Health. | BATTLECRY-TRIBE-BUFF |
| 42 | Junkbot | Mech | 5 | UNVERIFIED | Whenever a friendly Mech dies, gain +2/+2. | ON-DEATH-OF-TRIBE-SELF-BUFF |
| 43 | Floating Watcher | Demon | 3 | UNVERIFIED | Whenever your hero takes damage, gain +2/+2. | ON-DAMAGE-TAKEN(HERO)-SELF-BUFF |

(43 numbered rows above; one candidate — a Quilboar "gains stats when a friendly minion
survives lethal damage" card recalled as "Painsmith" — could not be verified this session
and was dropped rather than guessed. Sample: **43 cards**, spanning all 10 tribes plus
neutrals and all 6 tiers.)

---

## 2. Ranked template table

Templates ranked by number of sampled cards they cover (out of 43).

| Rank | Template | Shape | Count | % of sample |
|---|---|---|---|---|
| 1 | **TOKEN-SUMMON-ON-TRIGGER** | On {Battlecry \| Deathrattle \| take-damage}: summon N copies of token T | 8 | 19% |
| 2 | **BATTLECRY-TRIBE-BUFF** | Battlecry: give {a \| all \| other Mechs} +A/+H and/or a keyword | 6 | 14% |
| 3 | **TRIBE-SUMMON-TRIGGER-BUFF** | Whenever you summon tribe X: give {the summon \| self} +A/+H and/or a keyword | 4 | 9% |
| 4 | **GLOBAL-RULE-MODIFIER** | "Your {Battlecries \| Deathrattles \| summons} happen twice" | 3 | 7% |
| 5 | **END-OF-TURN-BUFF** | At end of turn, give {random \| tribe X} minion +A/+H permanently | 2 | 5% |
| 5 | **ON-DEATH-OF-TRIBE-SELF-BUFF** | Whenever a tribe X minion dies, gain +A/+H | 2 | 5% |
| 5 | **TURN-SCOPED-COUNTER-SCALED-BUFF** | Magnitude = a turn-scoped counter unrelated to the board | 2 | 5% |
| 8 | (7 singleton templates — see catalogue) | — | 1 each | 2% each |
| — | *(the 12 fully RESISTING cards, one template each — see §3)* | — | 13 | 30% |

Several "resisting" cards are counted once each above rather than folded into a false
template — that's the point of the exercise.

### Cumulative coverage curve

| Top N templates | Cards covered | % of sample (43) |
|---|---|---|
| Top 5 | 23 | 53% |
| Top 10 | 30 | 70% |
| Top 15 | 35 | 81% |

The curve flattens fast: past rank ~10 it's mostly singleton shapes, several of which are
themselves partly or fully resisting.

---

## 3. Cards that resist a clean, reusable, purely-parameterized template

For each: the specific capability a simple `{trigger, target, magnitude}` triple cannot
express.

1. **Baron Rivendare / Khadgar / Brann Bronzebeard** — each rewrites how *other* cards'
   abilities resolve (deathrattles, summons, battlecries triggering twice), not applying
   an effect of their own. Needs a **global interception of the relevant resolution
   step**, applying to cards not yet written.
2. **Murozond** — "Get a plain copy of a minion from your last opponent's warband."
   Needs to **read another player's board state from a previous combat round** — data
   gone from live game state by the time this resolves.
3. **Kangor's Apprentice** — "Summon your first 2 Mechs that died this combat." Needs an
   **ordered, filtered death log for the current combat**, built incrementally and read
   back retroactively.
4. **Southsea Strongarm / Slitherspear** — magnitude scales off a **turn-scoped counter
   from outside combat** (Pirates bought this turn; distinct spell types cast), living
   entirely in the shop/economy layer.
5. **Herald of Flame / Wildfire Elemental** — need an **Overkill trigger** (damage
   exceeding lethal, from inside the damage-math step) plus, for Herald, a **positional
   target** ("left-most enemy") recomputed as minions die.
6. **Waxrider Togwaggle** — "Whenever a friendly Dragon kills an enemy, gain +2/+2."
   Needs **kill attribution**: which friendly minion (and its tribe) caused a given
   death.
7. **Cave Hydra** — cleave onto minions adjacent to the attack target. Needs live
   **adjacency**, recomputed as neighbours die mid-combat.
8. **Goldrinn, the Great Wolf** — "For the rest of this combat, your Beasts have
   +12/+12." Needs a **third buff-duration class**: neither permanent nor a recomputed
   aura, but scoped to the current combat only.
9. **Tough Tusk** — trigger keyed to a specific secondary resource (Blood Gems), not a
   generic game event — the vocabulary would need to know that subsystem specifically.
10. **Replicating Menace** *(Magnetic)* — beyond its plain deathrattle, Magnetic is a
    **placement-dependent alternate play rule**, changing what "playing this card" means
    depending on where you drop it.

**13 of 43 sampled cards (30%)** need one of these extra capabilities; several (Herald of
Flame, Wildfire Elemental, Cave Hydra, Goldrinn) need more than one at once — the hardest
cards tend to need *combinations*, not just one extra primitive each.

---

## 4. Cross-cutting mechanics the per-card view misses

- **Recomputed auras are rarer than expected.** Most "X gets stronger" cards
  (Scavenging Hyena, Pack Leader, Mama Bear, Iron Sensei, Wrath Weaver, Crowd Favorite,
  Junkbot, Floating Watcher, King Bagurgle, Rockpool Hunter, Houndmaster, Metaltooth
  Leaper, Coldlight Seer) are **discrete, triggered, permanent stat grants**, not
  continuously-recomputed static auras. Goldrinn is the one clear exception (a temporary,
  combat-scoped buff). BG mostly avoids the hardest aura problem — but "permanent" and
  "combat-scoped" both need to exist as distinct primitives.
- **Triggers fire off other triggers' consequences.** Imp Gang Boss and Security Rover
  summon on taking damage; Deflect-o-Bot and Cobalt Guardian react to any Mech summon,
  including ones just created mid-combat by another trigger. The engine needs **trigger
  chains of arbitrary depth**, in a well-defined order, with a guard against runaway
  loops.
- **Board adjacency is load-bearing, not decorative.** Cave Hydra, Wildfire Elemental,
  and Herald of Flame all need "who's next to whom" or "who's at the edge" as a
  first-class, dynamically recomputed relationship, since the board shrinks and reorders
  constantly during combat.
- **State survives phase boundaries differently per card.** Permanent buffs (Pack
  Leader, Iron Sensei, Deflect-o-Bot) carry into the next Prep Phase; Goldrinn's
  explicitly doesn't. A data vocabulary needs at least two persistence semantics for "a
  minion has +A/+H."
- **Global modifiers change how other cards' data is interpreted.** Brann, Khadgar, and
  Baron Rivendare are simple one-liners, but supporting them means any other card's
  Battlecry/summon/Deathrattle resolution must first check for an active doubling
  modifier — a resolution-pipeline concern above the per-card effect vocabulary.
- **Turn/combat-scoped counters exist outside the board entirely.** Southsea Strongarm
  and Slitherspear both need small pieces of derived state, reset every turn, unrelated
  to any minion's stats — a parallel bookkeeping layer.

---

## 5. Bottom line

In this 43-card, all-tier, all-tribe sample, roughly **70% of cards** reduce to data
instances of about ten reusable trigger/target/magnitude templates, climbing to **~80%**
with a longer tail of one-off-but-simple templates (top 15). The remaining **~20–30%**
clusters around a small number of *capabilities*, not scattered special cases, in rough
order of how much coverage each would unlock:

1. **Dynamically-recomputed board adjacency** — needed by cleave, overkill-spillover,
   and positional-target cards.
2. **Access to combat-resolution internals** — overkill/excess damage and
   kill-attribution, both inside the damage-math step, not a simple "on kill" event.
3. **A resolution-pipeline interception point for global rule modifiers** ("your X
   happen twice") — every other card's resolution path must consult board-wide state
   before firing.
4. **More than one buff-duration class** — at minimum "permanent" and "for the rest of
   this combat"; true continuously-recomputed auras are rare in this pool.
5. **Read access to state outside the live board** — turn-scoped counters, an ordered
   per-combat death log, and a cross-round opponent snapshot.

Everything else in the sample — the large majority, including nearly every tribe's
defining synergy piece (Rat Pack, Pack Leader, Scavenging Hyena, Imp Gang Boss, King
Bagurgle, Deflect-o-Bot, Iron Sensei, Sellemental) — reduces cleanly to "on trigger T,
apply effect E to selector S, scaled by magnitude M," drawn from a modest fixed
vocabulary.
