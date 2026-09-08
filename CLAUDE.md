# HS-Battlegrounds-demake

A gameplay vertical slice of Hearthstone: Battlegrounds — Rust backend, TypeScript
frontend — with a number of diverging features/configurations: substantial, but not
meant to break the source game's model. Nine of those deltas are settled so far (see
[vision.md](docs/design/vision.md)).

**Fresh instance? Read, in order:** [`CONTEXT.md`](CONTEXT.md) (glossary),
[`docs/design/vision.md`](docs/design/vision.md) (the concept, and the deltas from it),
[`docs/design/roadmap.md`](docs/design/roadmap.md) (what "done" means), then
[`docs/scratchpad/state.md`](docs/scratchpad/state.md) (what's actually built, right
now). Skim [`docs/adr/`](docs/adr/) after. ~15 minutes, the whole picture.

## Working agreement

**Ethan designs and manages; Claude engineers.** Game-affecting decisions are never made
silently. Ethan's own framing, from the kickoff: *"I'm primarily interested in serving
the role of the designer & project manager, and I'm happy to grant you the role of the
engineer"* ([0001](docs/transcripts/0001-project-kickoff.md)).

- **Technical decisions go to Ethan as multiple-choice**, with a recommendation —
  *"I'd like to be in the loop for the technical decisions - so maybe those could be
  multiple choice modals with the options and your recommendation"*
  ([0001](docs/transcripts/0001-project-kickoff.md)).
- **Design questions are worked in rounds** via the `grilling` skill: the whole frontier
  at once, numbered, each with a recommendation, then wait — *"When you're done I'll ask
  you to 'grill me with docs' for the design"* ([0001](docs/transcripts/0001-project-kickoff.md)).
- **Facts are Claude's job** — dispatch a subagent rather than asking Ethan to look
  something up. (Established by example, not instruction: in
  [0001](docs/transcripts/0001-project-kickoff.md) Claude sent the card-shape survey to
  a subagent unprompted, rather than asking Ethan to estimate the number.)
- **Delegate mechanical, high-volume work to Sonnet subagents** — *"you should delegate
  to Sonnet Subagents when useful, in the interest of token efficiency"*
  ([0001](docs/transcripts/0001-project-kickoff.md)).
- **Give regular text updates on progress and thoughts** — *"Throughout this project,
  please give me regular text updates on your progress and thoughts as you go"*
  ([0001](docs/transcripts/0001-project-kickoff.md)).
- Update `CONTEXT.md` the moment a term settles; write an ADR only for hard-to-reverse,
  surprising, real trade-offs. In service of what Ethan actually asked for: docs and a
  workspace solid enough that *"even an amnesiac instance could easily jump back into the
  work"* ([0001](docs/transcripts/0001-project-kickoff.md)) — the same reason the
  [transcript archive](docs/transcripts/) exists at all.

Ethan's plugin marketplace is registered in
[`.claude/settings.json`](.claude/settings.json) but doesn't load in cloud sessions —
read `codebase-design`, `domain-modeling`, and `grilling` from a clone if needed.

## Where things are

| Path | What it is |
|---|---|
| `crates/bg-sim/` | The engine. Pure rules; no I/O. Deterministic given a Seed. |
| `crates/bg-cli/` | An empty stub, scaffolded early as a placeholder frontend crate name. Not *the* frontend — see [vision.md's concept section](docs/design/vision.md#the-concept) and [the scratchpad](docs/scratchpad/state.md) for the open question of its role now that the frontend is stated as TypeScript. |
| `CONTEXT.md` | Glossary. No implementation detail. |
| `docs/design/` | The concept, the deltas, and the roadmap. |
| `docs/adr/` | Numbered engineering decisions, with reasoning — Claude's, not Ethan's; see [its README](docs/adr/README.md) for what that means for claims inside them. |
| `docs/scratchpad/` | Claude's own working notes — current status, open engineering questions. Nothing here is ratified or binding. |
| `docs/research/` | Investigation findings, kept so they aren't re-run. |
| `docs/transcripts/` | Exports of the design conversations — the source of record for every quote cited elsewhere in these docs. |
| `scripts/` | Tooling outside the build. |

`bg-sim` doesn't depend on `bg-cli` and can't do I/O — "deep backend, shallow frontend"
made structural (Ethan's own phrase, from the kickoff message).

## Commands

```sh
cargo test                  # everything
cargo test -p bg-sim        # just the engine
cargo clippy --all-targets  # lints
cargo fmt --all             # format
python3 scripts/export_transcript.py --list   # available session transcripts
```

## State of play

v0.1 is in progress: the Action Phase resolves, Abilities are next. The frontend
technology is now stated (TypeScript, 2026-09-08), but nothing beyond `bg-sim`'s Rust
engine is built.

Full status — what's built, what's next, and every open engineering question awaiting
Ethan's ratification — lives in
[`docs/scratchpad/state.md`](docs/scratchpad/state.md), not here: that file is meant to
be rewritten freely every session, and duplicating it in this one would just let the two
drift.

**The default rule:** undeltered, it works however Battlegrounds works — see the deltas
in [vision.md](docs/design/vision.md). Numbers start at Battlegrounds' values, anchored
on 1.

**Vocabulary is Ethan's.** Claude's inventions are placeholders until ratified. *Pool*
and *action* stay non-specific. *Card* is the visual representation of a Unit; the engine
never mentions it.
