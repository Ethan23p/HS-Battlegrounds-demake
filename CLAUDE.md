# HS-Battlegrounds-demake

A deterministic auto-battler engine in Rust — a minimalist core of Hearthstone:
Battlegrounds with several rules deliberately changed.

**If you are a fresh instance, read these three files before touching anything:**
[`CONTEXT.md`](CONTEXT.md) (the glossary — use these words),
[`docs/design/vision.md`](docs/design/vision.md) (what we're building and the five
deltas from Battlegrounds), and [`docs/design/roadmap.md`](docs/design/roadmap.md)
(what "done" means). Then skim [`docs/adr/`](docs/adr/) for decisions already made and
why. That is about fifteen minutes and it is the whole picture.

## Working agreement

**Ethan is the designer and project manager. Claude is the engineer.** Design and game
rules are Ethan's call; implementation is Claude's. Do not silently make a design
decision — if a choice affects how the game plays, it goes to him.

- **Technical decisions go to Ethan as multiple-choice**, with options and a
  recommendation. He wants to be in the loop on them, not to make them from scratch.
- **Design questions are worked in rounds** using the `grilling` skill's format: ask the
  whole frontier at once, numbered, each with a recommended answer, then wait.
- **Facts are Claude's job, never Ethan's.** If a question needs a fact from the
  environment, the codebase, or the world, dispatch a subagent — don't ask him to look it
  up. Decisions are his; research is not.
- **Delegate mechanical, high-volume work to Sonnet subagents** — surveys, data authoring,
  tooling — to keep the main context on design and the engine.
- Update `CONTEXT.md` the moment a term is settled, and write an ADR when a decision is
  hard to reverse, surprising, and the result of a real trade-off. Not otherwise.

Ethan's own plugin marketplace (`Ethan23p/ethans-plugins_Claude-Code`, adapted from Matt
Pocock's) is registered in [`.claude/settings.json`](.claude/settings.json). In cloud
sessions it does not load — plugins there sync from the claude.ai catalog instead — so
read the skills directly from a clone of that repo if you need them. `codebase-design`,
`domain-modeling` and `grilling` are the ones this project runs on.

## Where things are

| Path | What it is |
|---|---|
| `crates/bg-sim/` | The engine. Pure rules; no I/O, no printing, no clock. |
| `crates/bg-cli/` | The terminal frontend. Deliberately thin; currently a stub. |
| `CONTEXT.md` | Glossary. Opinionated. Devoid of implementation detail. |
| `docs/design/` | Vision and roadmap. |
| `docs/adr/` | Decisions, numbered, with the reasoning that produced them. |
| `docs/research/` | Findings from investigations, kept so they aren't re-run. |
| `docs/transcripts/` | Human-readable exports of the design conversations. |
| `scripts/` | Tooling that isn't part of the build. |

The two-crate split is the "deep backend, shallow frontend" requirement made structural:
`bg-sim` does not depend on `bg-cli` and has no way to perform I/O, so game rules
physically cannot leak into display code.

## Commands

```sh
cargo test                  # everything
cargo test -p bg-sim        # just the engine
cargo clippy --all-targets  # lints
cargo fmt --all             # format
python3 scripts/export_transcript.py --list   # available session transcripts
```

## State of play

**Built:** `bg-sim::rng` — the deterministic generator (see
[ADR 0001](docs/adr/0001-own-the-random-number-generator.md)), 16 tests passing.
`bg-sim::cards` — a first draft of the ability vocabulary, **provisional**.

**Decided:** 1v1 rather than eight Seats; the Action Phase is a simulation of Beats rather
than a sequence of actions; Matches are asynchronous, with an unbounded Prep Phase and an
opponent drawn in advance from an Opponent Pool; three loosely-coupled in-run resources
(Economy, Power, Minions), bounded rather than strictly conserved; an original minimal card
set; headless-first with a human frontend after; v0.1 = an Action Phase resolves,
v0.2 = a Round completes, v0.3 = a Match completes. See [`docs/adr/`](docs/adr/).

**Open, and blocking:**

1. **Whether every Minion acts every Beat, or Minions have rates and a Beat advances a
   clock.** This gates v0.1 and everything downstream of it, including what Taunt and
   Windfury can mean.
2. **How Effects are expressed** — pure data vs. Rust per card vs. a hybrid with a
   pipeline hook. See the [card survey](docs/research/card-shape-survey.md): ~30% of a
   real card pool resists pure parameterisation, clustering on five named capabilities.
   Waiting on (1), since rates would add rate-modifying Effects to the vocabulary.
3. **What Economy, Power and the third axis precisely are**, what conversion between them
   costs, and the shape of the tax and loss-limit curves.
4. **Where Opponent Pool Boards come from**, and whether a Match stays a two-Seat contest
   with a winner or becomes a Run surviving a stream of drawn opponents.

**Vocabulary is Ethan's.** He supplies the words; Claude's inventions are placeholders
until he ratifies them. `CONTEXT.md` marks unratified proposals **(provisional)**. Three
collisions are currently unresolved there — most importantly that the resource axis called
"Minions" collides with `Minion`, the entity.

**Known inconsistency:** `crates/bg-sim/src/cards.rs` uses `CardId` and talks about
"cards", which `CONTEXT.md` rejects in favour of *Minion Definition*. It also names things
after "Combat", now the *Action Phase*. The rename is deliberately deferred: that module's
shape depends on open decision (2), so renaming now would be churn. Fix it when that lands.
