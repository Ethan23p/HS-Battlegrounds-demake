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
  ([ADR 0008](../adr/0008-targeting-is-random-simultaneity-is-the-only-delta.md)). Two
  things about a fight are actually ours: both sides act in the same Beat instead of
  alternating, and the attacker dies last on a mutual trade. Nothing ever attacks a
  Player — that's Hearthstone, not Battlegrounds. `rng` already draws from
  `Domain::Combat`, ahead of Effects needing it.
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

1. **Reborn's exact return stats.** Currently: keeps current attack, returns at 1 health,
   loses Reborn. Not verified against Battlegrounds; the default rule says Battlegrounds
   should settle it.
2. **Damage-on-loss.** Deferred to v0.2/v0.3, computed from `Resolution::final_board`'s
   survivors once Health exists as a Prep-Phase concept. The Action Phase itself no
   longer produces it — removed in
   [ADR 0008](../adr/0008-targeting-is-random-simultaneity-is-the-only-delta.md), which
   corrected an earlier overreach (see that ADR's "What this reverses").
3. **`bg-cli`'s role, now that the frontend is stated as TypeScript.** `bg-cli` was
   Claude's own scaffolding guess for "the frontend" (*"I made those names up"* —
   [0001](../transcripts/0001-project-kickoff.md)), not a decision Ethan made, and it's
   still an empty stub (`fn main() {}`). Ethan's 2026-09-08 concept statement is the
   first time a frontend technology has actually been named — see
   [vision.md's concept section](../design/vision.md#the-concept). Whether `bg-cli`
   becomes a debug/headless harness, an API surface the TypeScript frontend talks to, or
   gets deleted outright is undecided. Doesn't block v0.1 — still headless-first.
4. **Attacker-cycling and Windfury-against-an-empty-board.** Two implementation details
   Claude had to invent inside
   [ADR 0008](../adr/0008-targeting-is-random-simultaneity-is-the-only-delta.md): each
   side cycles left-to-right through its own Party for its attacker (recomputed fresh
   each Beat rather than a stored pointer, to dodge Battlegrounds' known pointer-quirk
   edge cases), and a Windfury Unit's second swing hits nothing rather than the Player if
   the opposing Party is already empty. Flagged in that ADR for correction; not yet
   explicitly ratified either way.
5. **"Pass" is still provisional** (see `CONTEXT.md`). Ethan's own phrase was "a round of
   beats"; *Round* was already taken. Rename freely if a better word turns up.

## Fixed in the 2026-09-08 docs overhaul

- `docs/transcripts/README.md` linked to `docs/adr/0010-the-clock-is-a-beat-counter.md`,
  which was never actually written — the beat-counter rewrite it refers to landed inside
  ADR 0009 instead. Link corrected to point there.
