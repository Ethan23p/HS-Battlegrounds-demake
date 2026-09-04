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
week's exact 300-card roster. Cards I could not verify via search this session are marked
**UNVERIFIED** and are flagged as drawn from memory; treat their exact wording with more
skepticism than the verified entries. No code in the repository was read, written, or
run for this task.

---

## 1. Card-by-card catalogue

Each row: Tribe / Tier (tier is approximate for a few cards — search snippets sometimes
disagreed with each other on tier, text was prioritized over tier), verified text, and
the template it was classified into.

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
and was dropped rather than guessed, per instructions. Sample size: **43 cards**, close
to but slightly under the 40–50 target; still spans all 10 tribes plus neutrals and all
6 tiers.)

---

## 2. Ranked template table

Templates ranked by number of sampled cards they cover (out of 43).

| Rank | Template | Shape | Count | % of sample |
|---|---|---|---|---|
| 1 | **TOKEN-SUMMON-ON-TRIGGER** | On {Battlecry \| Deathrattle \| take-damage}: summon N copies of token T (N fixed or = own current Attack) | 8 | 19% |
| 2 | **BATTLECRY-TRIBE-BUFF** | Battlecry: give {a friendly minion of tribe X \| all friendly minions of tribe X \| other Mechs} +A/+H and/or a keyword | 6 | 14% |
| 3 | **TRIBE-SUMMON-TRIGGER-BUFF** | Whenever you summon a minion of tribe X (during combat, for some cards): give {the summoned minion \| self} +A/+H and/or a keyword | 4 | 9% |
| 4 | **GLOBAL-RULE-MODIFIER** | "Your {Battlecries \| Deathrattles \| summon effects} happen twice" | 3 | 7% |
| 5 | **END-OF-TURN-BUFF** | At the end of your turn, give {a random friendly minion \| another friendly minion of tribe X} +A/+H permanently | 2 | 5% |
| 5 | **ON-DEATH-OF-TRIBE-SELF-BUFF** | Whenever a friendly minion of tribe X dies, gain +A/+H | 2 | 5% |
| 5 | **TURN-SCOPED-COUNTER-SCALED-BUFF** | Effect magnitude = a turn-scoped counter unrelated to the board (Pirates bought this turn; distinct spell types cast this turn) | 2 | 5% |
| 8 | PLAY-TRIGGER-TRIBE-SELF-EFFECT | After you play a minion of tribe X, apply a self-effect (damage self, buff self) | 1 | 2% |
| 8 | PLAY-TRIGGER-BY-KEYWORD-SELF-BUFF | Whenever you play a card with keyword K, gain +A/+H | 1 | 2% |
| 8 | START-OF-COMBAT-TRIBE-SCALED-DAMAGE | Start of Combat: deal N damage (scaled by a friendly-tribe count) to a random enemy | 1 | 2% |
| 8 | ON-SELL-REWARD | When you sell this, gain a reward | 1 | 2% |
| 8 | DEATHRATTLE-RANDOM-TARGET-KEYWORD | Deathrattle: give a random friendly minion a keyword | 1 | 2% |
| 8 | DEATHRATTLE-SUMMON-RANDOM-FILTERED | Deathrattle: summon N random minions matching a tag filter (pool query, not a fixed token) | 1 | 2% |
| 8 | STATIC-KEYWORD-ONLY | The card is just an evergreen keyword with no additional logic | 1 | 2% |
| 8 | CLEAVE-ON-ATTACK | Also damages the minion(s) adjacent to whichever enemy this attacks | 1 | 2% |
| 8 | ON-DAMAGE-TAKEN(HERO)-SELF-BUFF | Whenever your hero takes damage, gain +A/+H | 1 | 2% |
| — | *(the 12 fully RESISTING cards, one template each — see §3)* | — | 13 | 30% |

Note: several "resisting" cards are counted once each above and are broken out in the
resisting list below rather than folded into a false template — that's the point of the
exercise.

### Cumulative coverage curve

| Top N templates | Cards covered | % of sample (43) |
|---|---|---|
| Top 5 | 23 | 53% |
| Top 10 | 30 | 70% |
| Top 15 | 35 | 81% |

(Ranking ties broken by the order listed in the table above. The curve flattens fast:
the long tail past rank ~10 is mostly singleton shapes, several of which are themselves
partly or fully resisting.)

---

## 3. Cards that resist a clean, reusable, purely-parameterized template

For each: the specific mechanical capability the card needs that a simple
`{trigger, target, magnitude}` triple cannot express.

1. **Baron Rivendare** — "Your minions' Deathrattles trigger twice." Needs to **rewrite
   how every other card's ability resolves**, not apply an effect of its own. The
   deathrattle-resolution procedure itself must consult board-wide modifier state.
2. **Khadgar** — "Your cards that summon minions summon twice as many." Same category:
   a **global interception of the summon-resolution step**, applying to any card that
   summons, including ones not yet written.
3. **Brann Bronzebeard** — "Your Battlecries trigger twice." Same category again: a
   **global interception of the battlecry-resolution step**.
4. **Murozond** — "Get a plain copy of a minion from your last opponent's warband."
   Needs to **read another player's board state as it existed at the end of a previous
   combat round** — data that has left the live game state by the time this card
   resolves.
5. **Kangor's Apprentice** — "Summon your first 2 Mechs that died this combat." Needs an
   **ordered, tribe-filtered log of deaths that occurred earlier in the current combat**,
   queried retroactively at the moment this minion's own deathrattle fires. This is
   state that must be built up incrementally during combat resolution and then read
   back, not a value computed once.
6. **Southsea Strongarm** — "Repeat for each Pirate you bought this turn." Needs to read
   a **turn-scoped counter from the shop/economy layer** (purchases this turn) and use
   it as an effect's magnitude — the trigger condition lives entirely outside combat
   and outside the board.
7. **Slitherspear, Lord of Gains** — buff scaled "by each different spell you've cast
   this turn." Needs a **turn-scoped count of distinct spell *types* cast**, a derived
   value with its own dedup logic, not a simple counter increment.
8. **Herald of Flame** — "Overkill: Deal 3 damage to the left-most enemy minion." Needs
   (a) an **Overkill trigger**, which requires knowing how much damage exceeded the
   target's remaining health at the moment of a kill (a value from inside the combat
   damage-resolution math, not an event flag), and (b) a **positional target**
   ("left-most enemy") that must be recomputed as minions die during combat.
9. **Wildfire Elemental** — "deal excess damage to an adjacent enemy." Same excess/overkill
   value requirement as Herald of Flame, plus needs the **currently-adjacent minion**
   relative to a shrinking, reordering board.
10. **Waxrider Togwaggle** — "Whenever a friendly Dragon kills an enemy, gain +2/+2."
    Needs **kill attribution**: at the moment an enemy minion dies during combat, the
    engine must know *which* friendly minion's attack (and that minion's tribe) caused
    it, not just that a death occurred.
11. **Cave Hydra** — "Also damages the minions next to whomever this attacks." Needs
    live **adjacency**: "the minion(s) currently to the left/right of the target," which
    changes every time a minion on either side dies mid-combat.
12. **Goldrinn, the Great Wolf** — "For the rest of this combat, your Beasts have
    +12/+12." Needs a **third buff-duration class**: not a permanent stat grant (like
    almost every other buff card) and not a continuously-recomputed aura (BG mostly
    avoids those, see §4), but a buff that applies for the remainder of *this specific
    combat only* and must be discarded when combat ends, even though the minions
    receiving it persist into the next recruit phase unbuffed.
13. **Tough Tusk** — "After a Blood Gem is played on this, gain Divine Shield for the
    next combat." The trigger condition is not a generic game event but a specific
    secondary resource type (Blood Gems) with its own economy — the effect vocabulary
    would need to know about that resource subsystem specifically, not just "a spell
    was cast on this."
14. **Replicating Menace** *(Magnetic)* — beyond its plain deathrattle, Magnetic itself
    ("play this to the left of a friendly Mech to merge its stats and keywords into it")
    is a **placement-dependent alternate play rule**, not an effect that fires after the
    card resolves normally — it changes what "playing this card" even means depending on
    where you drop it and what's already there.

**13 of 43 sampled cards (30%)** need one of these extra capabilities. A few more
(Herald of Flame, Wildfire Elemental, Cave Hydra, Goldrinn) are double-counted across
categories 8–12 above because they need more than one capability at once — that overlap
itself is a data point: the hardest cards tend to need *combinations* (overkill +
position; kill-attribution + combat-scope), not just one extra primitive each.

---

## 4. Cross-cutting mechanics the per-card view misses

- **Auras that must be recomputed are rarer than you'd expect.** The overwhelming
  majority of "X gets stronger" cards in this sample (Scavenging Hyena, Pack Leader,
  Mama Bear, Iron Sensei, Wrath Weaver, Crowd Favorite, Junkbot, Floating Watcher, King
  Bagurgle, Rockpool Hunter, Houndmaster, Metaltooth Leaper, Coldlight Seer) are
  **discrete, triggered, permanent stat grants** — "on event E, add +A/+H to the
  minion's stats, once, forever" — not continuously-recomputed static auras ("while
  this is alive, other minions have +X"). Goldrinn is the one clear exception in this
  sample (a temporary, combat-scoped buff, not a recomputed aura either). This matters:
  it means the design space BG has actually chosen mostly avoids the hardest aura
  problem (dependency-graph recomputation), which is good news for a data vocabulary —
  but it means "permanent buff" and "combat-scoped buff" both need to exist as distinct
  primitives, and neither is a true continuous aura.
- **Triggers fire off other triggers' consequences.** Imp Gang Boss and Security Rover
  summon a minion when *they* take damage; Deflect-o-Bot and Cobalt Guardian react
  whenever *any* Mech is summoned, including one just created by Imp Gang Boss's or
  Replicating Menace's own trigger firing mid-combat. The engine needs to support
  **trigger chains of arbitrary depth** (a trigger's effect can itself satisfy another
  trigger's condition), in a well-defined order, with some guard against runaway loops —
  this is not visible from any single card's text, only from combinations.
- **Board position/adjacency is load-bearing, not decorative.** Cave Hydra (cleave),
  Wildfire Elemental (excess-damage spillover), and Herald of Flame (left-most target)
  all need "who is next to whom" or "who is at the edge" as a first-class, dynamically
  recomputed relationship — the board shrinks and reorders constantly during combat as
  minions die, so this can't be baked in once at combat start.
- **State must survive combat-phase boundaries differently depending on the card.**
  Permanent buffs earned during combat (Pack Leader's Beast buffs, Iron Sensei's Mech
  buffs, Deflect-o-Bot's stacked Attack/Divine Shield) carry forward into the next
  recruit phase and the rest of the game. Goldrinn's buff explicitly does **not** —
  "for the rest of this combat" means it must be wiped when combat ends even though nothing
  else about the minions changes. A data vocabulary needs at least two different
  persistence semantics for "a minion has +A/+H," not one.
- **Global modifiers change how other cards' data should be interpreted, not just what
  effects exist.** Brann, Khadgar, and Baron Rivendare are the sharpest version of this:
  they are themselves simple one-line cards, but supporting them means the code path
  that resolves *any other card's* Battlecry/summon/Deathrattle must first check "is
  there an active doubling modifier?" This is a resolution-pipeline concern that sits
  above the per-card effect vocabulary, not a fourth kind of effect card.
- **Turn-scoped and combat-scoped counters exist outside the board entirely.** Southsea
  Strongarm (Pirates bought this turn) and Slitherspear (distinct spells cast this turn)
  both need small pieces of derived state that reset every turn and have nothing to do
  with any minion's current stats — a parallel bookkeeping layer alongside "what's on
  the board."

---

## 5. Bottom line

In this 43-card, all-tier, all-tribe sample, roughly **70% of cards** are expressible as
data instances of about ten reusable trigger/target/magnitude templates (the "top 10"
column above), and coverage climbs to about **80%** if you're willing to add a longer
tail of one-off-but-still-simple templates (top 15). The remaining **~20–30%** clusters
tightly around a small number of *capabilities*, not a scattering of unrelated special
cases. In rough order of how much coverage each would unlock:

1. **A first-class, dynamically-recomputed board-adjacency relationship** ("the minion(s)
   next to X," "the left-most enemy") — needed by cleave, overkill-spillover, and
   positional-target cards, and it changes every time a minion dies mid-combat.
2. **Access to combat-resolution internals, not just combat events** — specifically
   *overkill/excess damage* (how much a killing blow exceeded lethal) and
   *kill-attribution* (which friendly minion's attack, of what tribe, caused a given
   death) — both live inside the damage-math step, not in a simple "on kill" event.
3. **A resolution-pipeline interception point for global rule modifiers** ("your X
   happen twice") — these are simple to describe but require every other card's
   ability-resolution code path to consult board-wide state before firing, which is a
   different kind of hook than "run effect E when event T happens."
4. **More than one buff-duration/persistence class** — at minimum, "permanent" and
   "for the rest of this combat only," since both patterns appear on staple cards and
   they behave differently at phase boundaries; true continuously-recomputed static
   auras are thankfully rare in the sampled pool.
5. **Read access to state outside the live board** — turn-scoped counters unrelated to
   minion stats (purchases this turn, distinct spells cast this turn), an ordered
   per-combat death log queryable retroactively, and a snapshot of an opponent's board
   from a previous round. These are less about *effect* vocabulary and more about what
   *queryable game state* exists for an effect to read.

Everything else in the sample — the large majority of cards, including nearly every
tribe's defining synergy piece (Rat Pack, Pack Leader, Scavenging Hyena, Imp Gang Boss,
King Bagurgle, Deflect-o-Bot, Iron Sensei, Sellemental) — reduces cleanly to "on trigger
T, apply effect E (buff/summon/keyword) to target selector S, scaled by magnitude M,"
with T, E, S, and M drawn from a modest fixed vocabulary.
