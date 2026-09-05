# HS-Battlegrounds-demake

A deterministic auto-battler engine in Rust: a vertical slice of Hearthstone: Battlegrounds
with five deliberate rule changes.

**Fresh instance? Read, in order:** [`CONTEXT.md`](CONTEXT.md) (glossary),
[`docs/design/vision.md`](docs/design/vision.md) (the five deltas),
[`docs/design/roadmap.md`](docs/design/roadmap.md) (what "done" means), then skim
[`docs/adr/`](docs/adr/). ~15 minutes, the whole picture.

## Working agreement

**Ethan designs and manages; Claude engineers.** Game-affecting decisions are never made
silently.

- **Technical decisions go to Ethan as multiple-choice**, with a recommendation.
- **Design questions are worked in rounds** via the `grilling` skill: the whole frontier
  at once, numbered, each with a recommendation, then wait.
- **Facts are Claude's job** — dispatch a subagent rather than asking Ethan to look
  something up.
- **Delegate mechanical, high-volume work to Sonnet subagents.**
- Update `CONTEXT.md` the moment a term settles; write an ADR only for hard-to-reverse,
  surprising, real trade-offs.

Ethan's plugin marketplace is registered in
[`.claude/settings.json`](.claude/settings.json) but doesn't load in cloud sessions —
read `codebase-design`, `domain-modeling`, and `grilling` from a clone if needed.

## Where things are

| Path | What it is |
|---|---|
| `crates/bg-sim/` | The engine. Pure rules; no I/O. |
| `crates/bg-cli/` | The terminal frontend. Thin; currently a stub. |
| `CONTEXT.md` | Glossary. No implementation detail. |
| `docs/design/` | Vision and roadmap. |
| `docs/adr/` | Numbered decisions, with reasoning. |
| `docs/research/` | Investigation findings, kept so they aren't re-run. |
| `docs/transcripts/` | Exports of the design conversations. |
| `scripts/` | Tooling outside the build. |

`bg-sim` doesn't depend on `bg-cli` and can't do I/O — "deep backend, shallow frontend"
made structural.

## Commands

```sh
cargo test                  # everything
cargo test -p bg-sim        # just the engine
cargo clippy --all-targets  # lints
cargo fmt --all             # format
python3 scripts/export_transcript.py --list   # available session transcripts
```

## State of play

**Built:** RNG ([ADR 0001](docs/adr/0001-own-the-random-number-generator.md)); the
ability vocabulary as data (provisional, Effects not yet executed); Units/Parties/Slots/
Board; and **the sweep**, `resolve(board) -> Resolution` — Windfury, Divine Shield,
Poisonous, Reborn live, Taunt inert pending its rule. 49 tests, clippy clean. `cargo run
-p bg-sim --example watch` prints a narrated fight.

The Action Phase uses **no randomness** — the sweep removed every decision that needed
it. `rng` waits for Effects.

**Decided:** one Player against a stream of opposing Parties; the Action Phase sweeps
Beats left-to-right, same-Slot Units resolving synchronously; 8 Slots per Party;
asynchronous Rounds, unbounded Prep; three loosely-coupled, bounded resources (Economy,
Power, Units); an original minimal Unit set; no heroes yet; headless-first; Abilities as
data through a pipeline with a modifier stage. See [`docs/adr/`](docs/adr/).

**The default rule:** undeltered, it works however Battlegrounds works — see the five
deltas in [vision.md](docs/design/vision.md). Numbers start at Battlegrounds' values,
anchored on 1.

**Next, to finish v0.1:** Abilities — Triggers, Effects through Selectors, cascading
Deathrattles, the modifier stage
([ADR 0007](docs/adr/0007-abilities-are-data-with-a-modifier-stage.md)). This is where
`rng` gets used and Unit data files get written.

**Open:**
1. Taunt's positional rule — unwritten, keyword inert until it exists.
2. Unopposed Units striking the Player — provisional, flagged for revisit.
3. Reborn's exact stats on return — unverified against Battlegrounds.

**Vocabulary is Ethan's.** Claude's inventions are placeholders until ratified. *Pool*
and *action* stay non-specific. *Card* is the visual representation of a Unit; the engine
never mentions it.
