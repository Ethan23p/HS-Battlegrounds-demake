# Project notes for Claude

## Fresh start (2026-09-11)

We are restarting the docs from `docs/DESIGN.md` (moved here from the repo root). Read
its opening declaration first: the source of truth is Ethan's own quotations, archived in
`docs/transcripts/`, and DESIGN.md is the organizational device that points at them.
Vision, then Design Decisions in two parts — Initial and Ongoing.

So `docs/transcripts/` is an archive to cite, not a doc to obey. Quoting Ethan from a
transcript is always fair; treating the surrounding Claude turns, or anything derived
from them, as settled design is not.

Everything else docs-shaped that predates the restart — `docs/research/`, `CONTEXT.md`'s
glossary, any ADRs or vocabulary referenced but not present (`docs/design/vision.md`,
`docs/adr/*`, terms like Instance/Pass/Answering as defined there) — is **prior-iteration
history, not current design**. Don't treat it as authoritative, don't assume it still
matches `docs/DESIGN.md`, and don't propagate its terminology just because it's used
consistently in old files. If something there still holds, it needs to be re-derived from
`docs/DESIGN.md` or confirmed with Ethan, not cited as-is.

When Ethan settles something new, add it to **Design Decisions → Ongoing** as his verbatim
quotation, in the order decided, with a one-line context note only where the quotation
would be incomplete without it — and a link to the transcript it came from. Run
`scripts/export_transcript.py` to archive the session first; see
`docs/transcripts/README.md`.

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
