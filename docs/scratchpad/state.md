# State

Claude's working notes on where the project actually stands. Rewrite this freely; it
carries no authority the way `CONTEXT.md` or an ADR does. See [README.md](README.md) for
the rules this directory follows.

## Built

- **RNG** — [ADR 0001](../adr/0001-own-the-random-number-generator.md). In-tree
  `xoshiro256**`, substreams per `Domain`.
- **Ability vocabulary as data** — provisional; Effects aren't executed yet.
- **Units / Parties / Slots / Board.**
- **The Action Phase** — `resolve(board, rng) -> Resolution`. Windfury, Divine Shield,
  Poisonous, Reborn, and Taunt all live. Targeting is random and respects Taunt, exactly
  as in Battlegrounds — every attack draws its own target
  ([ADR 0008](../adr/0008-targeting-is-random-simultaneity-is-the-only-delta.md)). An
  attack damages **both** Units: the target answers with its own attack in the same
  instant, and the indivisible step is the *instance*, not the Beat
  ([ADR 0010](../adr/0010-an-attack-is-an-exchange.md)). Two things about a fight are
  actually ours: both sides act in the same Beat instead of alternating, and the attacker
  dies last on a mutual trade. Nothing ever attacks a Player — that's Hearthstone, not
  Battlegrounds. `rng` already draws from `Domain::Combat`, ahead of Effects needing it.
- Clippy clean. `cargo run -p bg-sim --example watch` prints a narrated fight.

## Decided (see `docs/adr/` for the reasoning behind each)

One Player against a stream of opposing Parties; Beats are a time-step of the Board where
Beat *n* resolves Slot *n*, Beat 0 closing ranks
([ADR 0009](../adr/0009-the-party-is-left-anchored.md)); Parties are left-anchored,
closing ranks only at Beat 0; 8 Slots per Party; asynchronous Rounds, unbounded Prep;
three loosely-coupled bounded resources (Economy, Power, Units); an original minimal
Unit set; no heroes for the prototype; headless-first; Abilities as data through a
pipeline with a modifier stage
([ADR 0007](../adr/0007-abilities-are-data-with-a-modifier-stage.md)).

## Next, to finish v0.1

Abilities — Triggers, Effects through Selectors, cascading Deathrattles, the modifier
stage. Unit data files get written alongside.

## Open — awaiting Ethan's ratification or a design round

1. **`bg-cli`'s role, now that the frontend is stated as TypeScript.** `bg-cli` was
   Claude's own scaffolding guess for "the frontend" (*"I made those names up"* —
   [0001](../transcripts/0001-project-kickoff.md)), not a decision Ethan made, and it's
   still an empty stub (`fn main() {}`). Ethan's 2026-09-08 concept statement is the
   first time a frontend technology has actually been named — see
   [vision.md's concept section](../design/vision.md#the-concept). Whether `bg-cli`
   becomes a debug/headless harness, an API surface the TypeScript frontend talks to, or
   gets deleted outright is undecided. Doesn't block v0.1 — still headless-first.
2. **A Divine Shield absorbs the whole instant.** Chosen in
   [ADR 0010](../adr/0010-an-attack-is-an-exchange.md) because it is order-independent,
   which is the delta applied to its own consequences — but it is the *generous* reading,
   and "absorbs the largest blow, the rest land" is equally order-independent and
   stingier. Nothing else depends on the choice; one `land()` branch switches it. Worth a
   sentence from Ethan when the Unit set is big enough for the difference to be felt.
3. **"Pass", "instance" and "clash" are provisional** (see `CONTEXT.md`). Ethan's phrase
   for a Pass was "a round of beats"; *Round* was already taken. *Instance* and *clash*
   are Claude's, coined in ADR 0010. Rename freely if better words turn up.
4. **The Action Phase must not mutate the Party of record.** `resolve` takes the Board by
   value, so Rust already prevents a caller from seeing the fight's damage in its own
   Party — but nothing has tested it, because no Prep Phase exists to hold a Party
   between Rounds. Battlegrounds resets; so should we. Revisit when v0.2 builds the
   thing that owns a Party across Rounds.

### Closed since 2026-09-08

- **Reborn's exact return stats** — verified against Battlegrounds rather than asked
  about: current attack, 1 health, keeps other keywords, loses Reborn. The engine already
  did exactly this.
- **Attacker-cycling and Windfury-against-an-empty-board**, ADR 0008's two flagged
  inventions. Both confirmed as Battlegrounds' own behaviour and left as built: each side
  cycles left-to-right through its own Party (recomputed fresh each Beat), and a Windfury
  Unit's second swing hits nothing rather than the Player.
- **The keyword set is closed at five for v0.1.** Frenzy, Avenge and Magnetic are Triggers,
  which the vocabulary already has; Cleave is ability text through `Selector::Adjacent`;
  Venomous is a genuine candidate that no content yet needs.
- **Damage-on-loss** stays where the roadmap puts it — v0.2/v0.3, computed from
  `Resolution::final_board`'s survivors once Health exists as a Prep-Phase concept.

## Design notes for unbuilt systems

Forward-looking expectations Claude has recorded while reasoning through ADRs, for
systems that don't exist yet (Effects, Selectors, the frontend). Not verified, not
ratified — just worth not losing before the code that would confirm or break them
exists. Moved here from ADR 0009, which was asserting them as settled consequences of a
decision rather than open predictions.

- **Summons and the Beat clock.** Once Effects land, a Token arriving in a Slot the clock
  has already passed should wait for the next Pass; one arriving ahead of the clock
  should act in this one. Expected to need no special case, given
  [ADR 0009](../adr/0009-the-party-is-left-anchored.md)'s rule — unverified, since
  nothing summons anything yet.
- **`Selector::Adjacent` should be cheap** once it exists, because adjacency holds still
  for a whole Pass (positions don't move mid-Pass — ADR 0009): Slot arithmetic against an
  arrangement that isn't moving, rather than something recomputed live. Doesn't exist
  yet; the [card survey](../research/card-shape-survey.md) is what flagged adjacency as a
  needed capability in the first place.
- **The event log as the frontend's source of truth.** The assumption behind logging
  `Compacted` and similar Events is that a future frontend animates directly off the
  event log rather than diffing Board snapshots. Reasonable given "deep backend, shallow
  frontend," but nothing has built a frontend against it yet to confirm it holds up — and
  the frontend is now stated as TypeScript, a technology that didn't exist as a stated
  choice when this assumption was first written.

## Fixed in the 2026-09-08 docs overhaul

- `docs/transcripts/README.md` linked to `docs/adr/0010-the-clock-is-a-beat-counter.md`,
  which was never actually written — the beat-counter rewrite it refers to landed inside
  ADR 0009 instead. Link corrected to point there.
- **ADR 0003 mixed Claude's invented mechanics into what read like a record of Ethan's
  request.** It was already marked "partially superseded" by ADR 0008, but the callout
  didn't say *whose* overreach it was, and the struck-through text was detailed enough to
  be mistaken for something he'd actually asked for. Ethan flagged this directly — the
  ADRs are entirely Claude-authored, and he'd been reading some of their claims as though
  they were his own words. Rewrote the callout and every struck-through bullet to say
  plainly "Claude's invention, not Ethan's request," and added
  [`docs/adr/README.md`](../adr/README.md) stating the same rule for the whole directory:
  an unquoted "Ethan wanted X" inside an ADR is Claude's reading, not a fact.
- **Ingested `codebase-design`, `domain-modeling`, and `grilling`** from Ethan's
  marketplace (`Ethan23p/ethans-plugins_Claude-Code`, cloned to
  `/home/user/ethan23p/ethans-plugins_claude-code` — no local copy exists in every
  container, so a fresh instance may need to re-clone). Confirms this project's ADR and
  `CONTEXT.md` conventions already match `domain-modeling`'s format docs; its ADR
  template favors much shorter entries (1-3 sentences, sections added only when they earn
  their place) than this repo's ADRs actually are — worth knowing as an available
  direction, not applied wholesale, since Ethan said he isn't interested in following the
  skills precisely.
- **Audited the remaining ADRs (0001, 0002, 0004-0007, 0009) for the same pattern as
  0003** — not invented mechanics this time, but confident "Consequences" bullets stating
  Claude's own downstream reasoning as settled fact, on decisions where the core call was
  often genuinely Ethan's (ADR 0009's left-anchoring, most notably). 0001, 0004, and 0007
  held up as direct, grounded reasoning and weren't changed. Trimmed or hedged the
  overreaching bits in 0002, 0005, 0006, and 0009; moved 0009's claims about unbuilt
  systems (summons timing, `Selector::Adjacent`, the event log as frontend source of
  truth) here rather than deleting them outright, per Ethan's steer that solid claims
  worth keeping for Claude's own mental model can live somewhere hedged instead of
  asserted in an ADR.
