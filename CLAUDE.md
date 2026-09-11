# Project notes for Claude

## Fresh start (2026-09-11)

We are restarting the docs from `docs/DESIGN.md` (moved here from the repo root). That
document is the current source of truth for vision, departures, and design decisions.

Everything else docs-shaped that predates it — `docs/research/`, `docs/transcripts/`,
`CONTEXT.md`'s glossary, any ADRs or vocabulary referenced but not present
(`docs/design/vision.md`, `docs/adr/*`, terms like Beat/Instance/Pass/Slot/Answering as
defined there) — is **prior-iteration history, not current design**. Don't treat it as
authoritative, don't assume it still matches `docs/DESIGN.md`, and don't propagate its
terminology just because it's used consistently in old files. If something there still
holds, it needs to be re-derived from `docs/DESIGN.md` or confirmed with Ethan, not
cited as-is.

The code in `crates/` is being kept, not restarted — but it was written against the old
docs' concepts and vocabulary. Expect mismatches with `docs/DESIGN.md` (e.g. old docs
describe left-anchoring, simultaneous beats, a specific resource model — confirm each
still applies rather than assuming). When working on the code:

- Don't assume a comment, name, or module boundary reflects current design just because
  it's already in the code — check it against `docs/DESIGN.md`.
- Rework or remove comments that describe old concepts once the code they describe is
  adapted; don't leave comments referencing terminology or mechanics that no longer
  apply.
- Prefer adapting existing code over rewriting from scratch where the underlying logic
  still holds — this is a fresh start for docs, not necessarily for code.
