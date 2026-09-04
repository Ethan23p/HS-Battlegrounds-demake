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

**Built:** `bg-sim::rng` — the deterministic generator
([ADR 0001](docs/adr/0001-own-the-random-number-generator.md)). `bg-sim::units` — a first
draft of the ability vocabulary, **provisional**. 16 tests, clippy clean.

**Decided:** one Player against a stream of opposing Parties, not a lobby; the Action Phase
is a left-to-right sweep of Beats with same-Slot Units resolving synchronously; 8 Slots per
Party; Rounds are asynchronous with an unbounded Prep Phase; three loosely-coupled in-run
resources (Economy, Power, Units), bounded rather than conserved; an original minimal Unit
set; headless-first; v0.1 = an Action Phase resolves, v0.2 = a Round completes,
v0.3 = a Run completes. See [`docs/adr/`](docs/adr/).

**The default rule:** where we have not deliberately changed something, it works however
Battlegrounds works. The five deltas in [vision.md](docs/design/vision.md) are exhaustive,
not indicative. Numbers start at Battlegrounds' values, anchored on 1 as the atom.

**Open, and blocking:**

1. **The sweep's details** — whether it repeats after Slot 8, when deaths apply, and what
   happens in a Slot only one side occupies. Gates v0.1.
2. **Taunt and Windfury need new meanings.** The sweep removed target choice, so Taunt has
   nothing to constrain, and there is no turn for Windfury to take twice.
3. **How Effects are expressed** — pure data vs. Rust per Unit vs. a hybrid with a pipeline
   hook. See the [card survey](docs/research/card-shape-survey.md): ~30% of a real pool
   resists pure parameterisation, clustering on five named capabilities.
4. **Whether heroes exist.** Never discussed; `units.rs` still carries a speculative
   `HeroDef` that predates the design conversation. The default rule says yes, since
   Battlegrounds has them.

**Vocabulary is Ethan's.** He supplies the words; Claude's inventions are placeholders
until ratified. Note *pool* and *action* are deliberately non-specific — qualify them in
context rather than reserving them. *Card* means the visual representation of a Unit, a
presentation concept the engine never mentions.
