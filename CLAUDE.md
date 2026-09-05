# HS-Battlegrounds-demake

A deterministic auto-battler engine in Rust — a minimalist core of Hearthstone:
Battlegrounds with several rules deliberately changed.

**Fresh instance? Read these three first:** [`CONTEXT.md`](CONTEXT.md) (glossary — use
these words), [`docs/design/vision.md`](docs/design/vision.md) (what we're building and
the five deltas), [`docs/design/roadmap.md`](docs/design/roadmap.md) (what "done" means).
Then skim [`docs/adr/`](docs/adr/) for decisions made and why. ~15 minutes, the whole
picture.

## Working agreement

**Ethan designs and manages; Claude engineers.** Design and game rules are Ethan's call;
implementation is Claude's. Never make a design decision silently — if a choice affects
how the game plays, it goes to him.

- **Technical decisions go to Ethan as multiple-choice**, with options and a
  recommendation.
- **Design questions are worked in rounds** via the `grilling` skill's format: the whole
  frontier at once, numbered, each with a recommended answer, then wait.
- **Facts are Claude's job.** Dispatch a subagent rather than asking Ethan to look
  something up. Decisions are his; research isn't.
- **Delegate mechanical, high-volume work to Sonnet subagents** to keep the main context
  on design and the engine.
- Update `CONTEXT.md` the moment a term is settled; write an ADR only when a decision is
  hard to reverse, surprising, and the result of a real trade-off.

Ethan's plugin marketplace (`Ethan23p/ethans-plugins_Claude-Code`) is registered in
[`.claude/settings.json`](.claude/settings.json). It doesn't load in cloud sessions
(plugins there sync from the claude.ai catalog) — read skills directly from a clone if
needed. `codebase-design`, `domain-modeling`, and `grilling` are the ones this project
runs on.

## Where things are

| Path | What it is |
|---|---|
| `crates/bg-sim/` | The engine. Pure rules; no I/O, no printing, no clock. |
| `crates/bg-cli/` | The terminal frontend. Deliberately thin; currently a stub. |
| `CONTEXT.md` | Glossary. Opinionated, no implementation detail. |
| `docs/design/` | Vision and roadmap. |
| `docs/adr/` | Decisions, numbered, with the reasoning behind them. |
| `docs/research/` | Findings from investigations, kept so they aren't re-run. |
| `docs/transcripts/` | Human-readable exports of the design conversations. |
| `scripts/` | Tooling outside the build. |

The two-crate split makes "deep backend, shallow frontend" structural: `bg-sim` can't
depend on `bg-cli` or perform I/O, so game rules can't leak into display code.

## Commands

```sh
cargo test                  # everything
cargo test -p bg-sim        # just the engine
cargo clippy --all-targets  # lints
cargo fmt --all             # format
python3 scripts/export_transcript.py --list   # available session transcripts
```

## State of play

**Built:**
- `bg-sim::rng` — the deterministic generator ([ADR 0001](docs/adr/0001-own-the-random-number-generator.md)).
- `bg-sim::units` — the ability vocabulary as data. Provisional; Effects aren't executed yet.
- `bg-sim::party` — Units, Parties, Slots, Board. Left-packed invariant at Sweep boundaries.
- `bg-sim::action_phase` — **the sweep, working.** `resolve(board) -> Resolution`.
  Windfury, Divine Shield, Poisonous, Reborn live; Taunt is inert pending its rule.

49 tests, clippy clean. `cargo run -p bg-sim --example watch` prints a narrated fight.

The Action Phase uses **no randomness** — the positional sweep removed every decision
that needed it. `rng` waits for Effects, the first thing that will need it.

**Decided:** one Player against a stream of opposing Parties, not a lobby; the Action
Phase is a left-to-right sweep of Beats, same-Slot Units resolving synchronously; 8 Slots
per Party; asynchronous Rounds with an unbounded Prep Phase; three loosely-coupled,
bounded-not-conserved resources (Economy, Power, Units); an original minimal Unit set; no
heroes in the prototype; headless-first; Abilities as data through a pipeline with a
modifier stage. See [`docs/adr/`](docs/adr/).

**The default rule:** where we haven't deliberately changed something, it works however
Battlegrounds works. The five deltas in [vision.md](docs/design/vision.md) are exhaustive,
not indicative. Numbers start at Battlegrounds' values, anchored on 1 as the atom.

**Next, to finish v0.1:** Abilities — Triggers firing, Effects resolving through
Selectors, cascading Deathrattles, and the modifier stage from
[ADR 0007](docs/adr/0007-abilities-are-data-with-a-modifier-stage.md). This is where
`rng` finally gets used and the Unit data files get written.

**Open:**

1. **Taunt's positional rule** is unwritten; the keyword stays inert until it exists.
2. **Unopposed Units striking the Player** is provisional — flagged for revisit.
3. **Reborn's exact stats on return** (currently attack + 1 health, losing Reborn) is
   unverified against Battlegrounds.

**Vocabulary is Ethan's.** He supplies the words; Claude's inventions are placeholders
until ratified. *Pool* and *action* are deliberately non-specific. *Card* means the
visual representation of a Unit — the engine never mentions it.
